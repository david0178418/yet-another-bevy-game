use bevy::prelude::*;
use std::f32::consts::PI;

pub struct WeaponsPlugin;

const BLADE_RADIUS: f32 = 80.0;
const BLADE_SPEED: f32 = 3.0;
const BLADE_DAMAGE: f32 = 200.0;
const BLADE_SIZE: Vec2 = Vec2::new(20.0, 10.0);
const BLADE_COLOR: Color = Color::srgb(0.8, 0.8, 0.9);

const SHOOTER_COOLDOWN: f32 = 1.5;
const SHOOTER_DAMAGE: f32 = 250.0;
const SHOOTER_PROJECTILE_SPEED: f32 = 300.0;

const PROJECTILE_SIZE: Vec2 = Vec2::new(15.0, 8.0);
const PROJECTILE_COLOR: Color = Color::srgb(0.9, 0.9, 0.3);
const PROJECTILE_LIFETIME: f32 = 3.0;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            update_orbiting_blade,
            update_auto_shooter,
            move_projectiles,
        ));
    }
}

#[derive(Component)]
pub struct OrbitingBlade {
    pub angle: f32,
    pub radius: f32,
    pub speed: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct AutoShooter {
    pub cooldown: Timer,
    pub damage: f32,
    pub projectile_speed: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: Timer,
}

pub fn spawn_orbiting_blade(
    commands: &mut Commands,
    count: u32,
    blade_query: &mut Query<&mut OrbitingBlade>,
) {
    let existing_count = blade_query.iter().count();
    let total_count = existing_count + count as usize;

    // Redistribute existing blades
    for (index, mut blade) in blade_query.iter_mut().enumerate() {
        blade.angle = (index as f32 / total_count as f32) * 2.0 * PI;
    }

    // Spawn new blades with evenly distributed angles
    for i in 0..count {
        let blade_index = existing_count + i as usize;
        commands.spawn((
            Sprite {
                color: BLADE_COLOR,
                custom_size: Some(BLADE_SIZE),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            OrbitingBlade {
                angle: (blade_index as f32 / total_count as f32) * 2.0 * PI,
                radius: BLADE_RADIUS,
                speed: BLADE_SPEED,
                damage: BLADE_DAMAGE,
            },
        ));
    }
}

pub fn spawn_auto_shooter(commands: &mut Commands) {
    commands.spawn(AutoShooter {
        cooldown: Timer::from_seconds(SHOOTER_COOLDOWN, TimerMode::Repeating),
        damage: SHOOTER_DAMAGE,
        projectile_speed: SHOOTER_PROJECTILE_SPEED,
    });
}

fn update_orbiting_blade(
    mut blade_query: Query<(&mut Transform, &mut OrbitingBlade)>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<OrbitingBlade>)>,
    time: Res<Time<Virtual>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut blade_transform, mut blade) in blade_query.iter_mut() {
            blade.angle += blade.speed * time.delta_secs();

            blade_transform.translation.x = player_transform.translation.x + blade.angle.cos() * blade.radius;
            blade_transform.translation.y = player_transform.translation.y + blade.angle.sin() * blade.radius;
            blade_transform.rotation = Quat::from_rotation_z(blade.angle + PI / 2.0);
        }
    }
}

fn update_auto_shooter(
    mut commands: Commands,
    mut shooter_query: Query<&mut AutoShooter>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    enemy_query: Query<&Transform, With<crate::enemy::Enemy>>,
    time: Res<Time<Virtual>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut shooter in shooter_query.iter_mut() {
            if shooter.cooldown.tick(time.delta()).just_finished() {
                // Find nearest enemy
                let mut nearest_distance = f32::MAX;
                let mut nearest_direction = 1.0;

                for enemy_transform in enemy_query.iter() {
                    let distance = player_transform.translation.distance(enemy_transform.translation);
                    if distance < nearest_distance {
                        nearest_distance = distance;
                        nearest_direction = (enemy_transform.translation.x - player_transform.translation.x).signum();
                    }
                }

                // Spawn projectile
                commands.spawn((
                    Sprite {
                        color: PROJECTILE_COLOR,
                        custom_size: Some(PROJECTILE_SIZE),
                        ..default()
                    },
                    Transform::from_xyz(
                        player_transform.translation.x + nearest_direction * 30.0,
                        player_transform.translation.y,
                        0.0,
                    ),
                    crate::physics::Velocity {
                        x: nearest_direction * shooter.projectile_speed,
                        y: 0.0,
                    },
                    Projectile {
                        damage: shooter.damage,
                        lifetime: Timer::from_seconds(PROJECTILE_LIFETIME, TimerMode::Once),
                    },
                ));
            }
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut projectile) in projectile_query.iter_mut() {
        if projectile.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
