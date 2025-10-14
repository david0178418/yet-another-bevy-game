use bevy::{ecs::system::ParamSet, prelude::*};

pub struct PhysicsPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionResolutionSet;

impl Plugin for PhysicsPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(apply_gravity, apply_velocity, check_ground_collision)
				.chain()
				.in_set(PhysicsSet),
		)
		.add_systems(
			Update,
			resolve_entity_collisions
				.in_set(CollisionResolutionSet)
				.after(PhysicsSet),
		);
	}
}

#[derive(Component)]
pub struct Velocity {
	pub x: f32,
	pub y: f32,
}

#[derive(Component)]
pub struct Grounded(pub bool);

/// Marker component for static/immovable objects like platforms.
/// Entities with Ground are excluded from dynamic collision resolution
/// but are still used for ground detection.
#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Collider;

type ColliderQuery<'w, 's> =
	Query<'w, 's, (Entity, &'static Transform, &'static Sprite), With<Collider>>;
type GroundedQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static mut Transform,
		&'static Sprite,
		&'static mut Velocity,
		&'static mut Grounded,
	),
>;
type CollisionQuery<'w, 's> =
	Query<'w, 's, (&'static mut Transform, &'static Sprite), (With<Collider>, Without<Ground>)>;

fn apply_gravity(
	mut query: Query<(&mut Velocity, &Grounded), Without<crate::behaviors::EnergyCharging>>,
	time: Res<Time<Virtual>>,
) {
	for (mut velocity, grounded) in query.iter_mut() {
		if !grounded.0 {
			velocity.y += crate::constants::GRAVITY * time.delta_secs();
		}
	}
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time<Virtual>>) {
	for (mut transform, velocity) in query.iter_mut() {
		transform.translation.x += velocity.x * time.delta_secs();
		transform.translation.y += velocity.y * time.delta_secs();
	}
}

fn resolve_entity_collisions(mut query: CollisionQuery) {
	let mut combinations = query.iter_combinations_mut();

	while let Some([(mut transform1, sprite1), (mut transform2, sprite2)]) =
		combinations.fetch_next()
	{
		let size1 = sprite1.custom_size.unwrap_or(Vec2::ONE);
		let size2 = sprite2.custom_size.unwrap_or(Vec2::ONE);

		let half_size1 = size1 / 2.0;
		let half_size2 = size2 / 2.0;

		let pos1 = transform1.translation;
		let pos2 = transform2.translation;

		let delta = pos2 - pos1;
		let min_distance = half_size1 + half_size2;

		let overlap_x = min_distance.x - delta.x.abs();
		let overlap_y = min_distance.y - delta.y.abs();

		if overlap_x <= 0.0 || overlap_y <= 0.0 {
			continue;
		}

		// Resolve collision by pushing apart on the axis with least overlap
		if overlap_x < overlap_y {
			// Separate on X axis
			let push = overlap_x / 2.0 * delta.x.signum();
			transform1.translation.x -= push;
			transform2.translation.x += push;
		} else {
			// Separate on Y axis
			let push = overlap_y / 2.0 * delta.y.signum();
			transform1.translation.y -= push;
			transform2.translation.y += push;
		}
	}
}

fn check_ground_collision(mut param_set: ParamSet<(ColliderQuery, GroundedQuery)>) {
	// First pass: collect all collider positions and sizes
	let collider_data: Vec<(Entity, Vec3, Vec2)> = param_set
		.p0()
		.iter()
		.map(|(entity, transform, sprite)| {
			(
				entity,
				transform.translation,
				sprite.custom_size.unwrap_or(Vec2::ONE),
			)
		})
		.collect();

	// Second pass: detect ground collisions and update grounded entities
	for (entity, mut entity_transform, entity_sprite, mut velocity, mut grounded) in
		param_set.p1().iter_mut()
	{
		let entity_size = entity_sprite.custom_size.unwrap_or(Vec2::ONE);
		let entity_bottom = entity_transform.translation.y - entity_size.y / 2.0;
		let entity_left = entity_transform.translation.x - entity_size.x / 2.0;
		let entity_right = entity_transform.translation.x + entity_size.x / 2.0;

		grounded.0 = false;

		for (collider_entity, collider_translation, collider_size) in &collider_data {
			// Skip self
			if entity == *collider_entity {
				continue;
			}

			let collider_top = collider_translation.y + collider_size.y / 2.0;
			let collider_left = collider_translation.x - collider_size.x / 2.0;
			let collider_right = collider_translation.x + collider_size.x / 2.0;

			// Skip if not overlapping horizontally
			if entity_right <= collider_left || entity_left >= collider_right {
				continue;
			}

			// Skip if not close to ground or moving upward
			if entity_bottom > collider_top
				|| entity_bottom <= collider_top - crate::constants::GROUND_SNAP_DISTANCE
				|| velocity.y > 0.0
			{
				continue;
			}

			grounded.0 = true;
			velocity.y = 0.0;
			entity_transform.translation.y = collider_top + entity_size.y / 2.0;
			break;
		}
	}
}
