use bevy::prelude::*;

pub struct MovementPlugin;

type EnemyTransformQuery<'w, 's> = Query<
	'w,
	's,
	(Entity, &'static Transform),
	(
		With<crate::behaviors::EnemyTag>,
		Without<crate::behaviors::MaintainDistance>,
	),
>;

impl Plugin for MovementPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				update_seek_target_entities,
				update_zigzag_entities,
				update_maintain_distance_entities,
			),
		);
	}
}

fn update_seek_target_entities(
	mut seek_query: Query<(
		&mut crate::physics::Velocity,
		&crate::behaviors::SeekTarget,
		Has<crate::behaviors::Stunned>,
	)>,
	player_query: Query<&Transform, With<crate::behaviors::PlayerTag>>,
	enemy_query: Query<(Entity, &Transform), With<crate::behaviors::EnemyTag>>,
) {
	use crate::behaviors::TargetType;

	for (mut velocity, seek, is_stunned) in seek_query.iter_mut() {
		if is_stunned {
			continue;
		}

		let target_position = match seek.target_type {
			TargetType::Player => player_query.single().ok().map(|t| t.translation),
			TargetType::NearestEnemy => enemy_query
				.iter()
				.min_by(|(_, a), (_, b)| {
					let dist_a = a.translation.length();
					let dist_b = b.translation.length();
					dist_a.partial_cmp(&dist_b).unwrap()
				})
				.map(|(_, t)| t.translation),
		};

		if let Some(target_pos) = target_position {
			let direction = Vec2::new(target_pos.x, target_pos.y).normalize_or_zero();
			velocity.x = direction.x * seek.speed;
			velocity.y = direction.y * seek.speed;
		}
	}
}

fn update_zigzag_entities(
	mut zigzag_query: Query<(
		&Transform,
		&mut crate::physics::Velocity,
		&mut crate::behaviors::ZigZagMovement,
		Has<crate::behaviors::Stunned>,
	)>,
	player_query: Query<
		&Transform,
		(
			With<crate::behaviors::PlayerTag>,
			Without<crate::behaviors::ZigZagMovement>,
		),
	>,
	time: Res<Time<Virtual>>,
) {
	if let Ok(player_transform) = player_query.single() {
		for (transform, mut velocity, mut zigzag, is_stunned) in zigzag_query.iter_mut() {
			if is_stunned {
				continue;
			}

			zigzag.time += time.delta_secs();

			let direction_to_player = Vec2::new(
				player_transform.translation.x - transform.translation.x,
				player_transform.translation.y - transform.translation.y,
			)
			.normalize_or_zero();

			let perpendicular = Vec2::new(-direction_to_player.y, direction_to_player.x);

			let oscillation =
				(zigzag.time * zigzag.oscillation_speed).sin() * zigzag.oscillation_amplitude;

			let final_direction =
				(direction_to_player + perpendicular * oscillation).normalize_or_zero();

			velocity.x = final_direction.x * zigzag.base_speed;
			velocity.y = final_direction.y * zigzag.base_speed;
		}
	}
}

fn update_maintain_distance_entities(
	mut maintain_query: Query<(
		&Transform,
		&mut crate::physics::Velocity,
		&crate::behaviors::MaintainDistance,
		Has<crate::behaviors::Stunned>,
	)>,
	player_query: Query<
		&Transform,
		(
			With<crate::behaviors::PlayerTag>,
			Without<crate::behaviors::MaintainDistance>,
		),
	>,
	enemy_query: EnemyTransformQuery,
) {
	use crate::behaviors::TargetType;

	for (transform, mut velocity, maintain, is_stunned) in maintain_query.iter_mut() {
		if is_stunned {
			continue;
		}

		let target_position = match maintain.target_type {
			TargetType::Player => player_query.single().ok().map(|t| t.translation),
			TargetType::NearestEnemy => enemy_query
				.iter()
				.min_by(|(_, a), (_, b)| {
					let dist_a = (a.translation - transform.translation).length();
					let dist_b = (b.translation - transform.translation).length();
					dist_a.partial_cmp(&dist_b).unwrap()
				})
				.map(|(_, t)| t.translation),
		};

		if let Some(target_pos) = target_position {
			let direction_to_target = Vec2::new(
				target_pos.x - transform.translation.x,
				target_pos.y - transform.translation.y,
			);
			let distance = direction_to_target.length();
			let normalized_direction = direction_to_target.normalize_or_zero();

			const DISTANCE_THRESHOLD: f32 = 10.0;

			if distance > maintain.preferred_distance + DISTANCE_THRESHOLD {
				velocity.x = normalized_direction.x * maintain.speed;
				velocity.y = normalized_direction.y * maintain.speed;
			} else if distance < maintain.preferred_distance - DISTANCE_THRESHOLD {
				velocity.x = -normalized_direction.x * maintain.speed;
				velocity.y = -normalized_direction.y * maintain.speed;
			} else {
				velocity.x = 0.0;
				velocity.y = 0.0;
			}
		}
	}
}
