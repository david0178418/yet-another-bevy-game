use bevy::prelude::*;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_gravity,
            apply_velocity,
            check_ground_collision,
        ).chain());
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

            // Check if player is above ground and overlapping horizontally
            if player_right > ground_left && player_left < ground_right {
                // Check if player is close to ground and moving downward
                if player_bottom <= ground_top && player_bottom > ground_top - crate::constants::GROUND_SNAP_DISTANCE && velocity.y <= 0.0 {
                    grounded.0 = true;
                    velocity.y = 0.0;
                    // Snap position to ground surface to prevent clipping
                    player_transform.translation.y = ground_top + player_size.y / 2.0;
                    break;
                }
            }
        }
    }
}
