use bevy::prelude::*;

pub struct PhysicsPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicsSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollisionResolutionSet;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_gravity,
            apply_velocity,
            check_ground_collision,
        ).chain().in_set(PhysicsSet))
        .add_systems(Update,
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

#[derive(Component)]
pub struct Ground;

#[derive(Component)]
pub struct Collider;

fn apply_gravity(
    mut query: Query<(&mut Velocity, &Grounded)>,
    time: Res<Time<Virtual>>,
) {
    for (mut velocity, grounded) in query.iter_mut() {
        if !grounded.0 {
            velocity.y += crate::constants::GRAVITY * time.delta_secs();
        }
    }
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time<Virtual>>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * time.delta_secs();
        transform.translation.y += velocity.y * time.delta_secs();
    }
}

fn resolve_entity_collisions(
    mut query: Query<(&mut Transform, &Sprite), With<Collider>>,
) {
    let mut combinations = query.iter_combinations_mut();

    while let Some([(mut transform1, sprite1), (mut transform2, sprite2)]) = combinations.fetch_next() {
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

fn check_ground_collision(
    mut player_query: Query<(&mut Transform, &Sprite, &mut Velocity, &mut Grounded), Without<Ground>>,
    ground_query: Query<(&Transform, &Sprite), With<Ground>>,
) {
    for (mut player_transform, player_sprite, mut velocity, mut grounded) in player_query.iter_mut() {
        let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);
        let player_bottom = player_transform.translation.y - player_size.y / 2.0;
        let player_left = player_transform.translation.x - player_size.x / 2.0;
        let player_right = player_transform.translation.x + player_size.x / 2.0;

        grounded.0 = false;

        for (ground_transform, ground_sprite) in ground_query.iter() {
            let ground_size = ground_sprite.custom_size.unwrap_or(Vec2::ONE);
            let ground_top = ground_transform.translation.y + ground_size.y / 2.0;
            let ground_left = ground_transform.translation.x - ground_size.x / 2.0;
            let ground_right = ground_transform.translation.x + ground_size.x / 2.0;

            // Skip if not overlapping horizontally
            if player_right <= ground_left || player_left >= ground_right {
                continue;
            }

            // Skip if not close to ground or moving upward
            if player_bottom > ground_top || player_bottom <= ground_top - crate::constants::GROUND_SNAP_DISTANCE || velocity.y > 0.0 {
                continue;
            }

            grounded.0 = true;
            velocity.y = 0.0;
            player_transform.translation.y = ground_top + player_size.y / 2.0;
            break;
        }
    }
}
