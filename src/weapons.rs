use bevy::prelude::*;
use std::f32::consts::PI;

pub struct WeaponsPlugin;

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
    _player_entity: Entity,
    count: u32,
) {
    for i in 0..count {
        commands.spawn((
            Sprite {
                color: Color::srgb(0.8, 0.8, 0.9),
                custom_size: Some(Vec2::new(20.0, 10.0)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            OrbitingBlade {
                angle: (i as f32 / count as f32) * 2.0 * PI,
                radius: 80.0,
                speed: 3.0,
                damage: 200.0,
            },
        ));
    }
}

pub fn spawn_auto_shooter(commands: &mut Commands, _player_entity: Entity) {
    commands.spawn(AutoShooter {
        cooldown: Timer::from_seconds(1.5, TimerMode::Repeating),
        damage: 250.0,
        projectile_speed: 300.0,
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
                        color: Color::srgb(0.9, 0.9, 0.3),
                        custom_size: Some(Vec2::new(15.0, 8.0)),
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
                        lifetime: Timer::from_seconds(3.0, TimerMode::Once),
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
