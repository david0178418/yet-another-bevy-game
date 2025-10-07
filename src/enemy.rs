use bevy::prelude::*;
use rand::Rng;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemySpawnTimer(Timer::from_seconds(2.0, TimerMode::Repeating)))
            .insert_resource(WaveTimer {
                timer: Timer::from_seconds(30.0, TimerMode::Repeating),
                wave: 1,
            })
            .add_systems(Update, (
                spawn_enemies,
                move_enemies,
                update_wave,
                enemy_death,
                update_health_bars,
            ));
    }
}

#[derive(Resource)]
struct EnemySpawnTimer(Timer);

#[derive(Resource)]
struct WaveTimer {
    timer: Timer,
    wave: u32,
}

#[derive(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub xp_value: u32,
}


#[derive(Component)]
struct HealthBar {
    enemy_entity: Entity,
}

#[derive(Component)]
struct HealthBarBackground;

#[derive(Component)]
struct HealthBarForeground {
    max_health: f32,
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time<Virtual>>,
    mut timer: ResMut<EnemySpawnTimer>,
    wave: Res<WaveTimer>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();

        if let Ok(player_transform) = player_query.get_single() {
            // Spawn enemies off-screen
            let spawn_side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            let spawn_x = player_transform.translation.x + spawn_side * 700.0;
            let spawn_y = rng.gen_range(-200.0..100.0);

            let enemy_type = rng.gen_range(0..3);

            let (color, health, speed, damage, size, xp) = match enemy_type {
                0 => (Color::srgb(0.8, 0.2, 0.2), 15.0, 80.0, 10.0, Vec2::new(30.0, 30.0), 5),  // Red - weak, fast
                1 => (Color::srgb(0.2, 0.8, 0.2), 30.0, 50.0, 15.0, Vec2::new(40.0, 40.0), 10), // Green - medium
                _ => (Color::srgb(0.8, 0.8, 0.2), 50.0, 30.0, 25.0, Vec2::new(50.0, 50.0), 20), // Yellow - strong, slow
            };

            let scaled_health = health * (1.0 + (wave.wave as f32 * 0.2));

            let enemy_entity = commands.spawn((
                Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y, 0.0),
                Enemy {
                    health: scaled_health,
                    speed,
                    damage,
                    xp_value: xp,
                },
                crate::physics::Velocity { x: 0.0, y: 0.0 },
                crate::physics::Grounded(false),
            )).id();

            // Spawn health bar background
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(size.x, 4.0)),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y + size.y / 2.0 + 8.0, 1.0),
                HealthBar { enemy_entity },
                HealthBarBackground,
            ));

            // Spawn health bar foreground
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.8, 0.0),
                    custom_size: Some(Vec2::new(size.x, 4.0)),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y + size.y / 2.0 + 8.0, 2.0),
                HealthBar { enemy_entity },
                HealthBarForeground { max_health: scaled_health },
            ));
        }
    }
}

fn move_enemies(
    mut enemy_query: Query<(&Transform, &mut crate::physics::Velocity, &Enemy), Without<crate::player::Player>>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (enemy_transform, mut velocity, enemy) in enemy_query.iter_mut() {
            let direction = (player_transform.translation.x - enemy_transform.translation.x).signum();
            velocity.x = direction * enemy.speed;
        }
    }
}

fn update_wave(
    mut wave: ResMut<WaveTimer>,
    time: Res<Time<Virtual>>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    if wave.timer.tick(time.delta()).just_finished() {
        wave.wave += 1;

        // Increase spawn rate with each wave (decrease time between spawns)
        let new_duration = (2.0 - (wave.wave as f32 * 0.1)).max(0.5);
        spawn_timer.0.set_duration(std::time::Duration::from_secs_f32(new_duration));
    }
}

fn enemy_death(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform, &Enemy)>,
    health_bar_query: Query<(Entity, &HealthBar)>,
) {
    for (entity, transform, enemy) in enemy_query.iter() {
        if enemy.health <= 0.0 {
            // Spawn experience orb
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.9, 0.7, 0.2),
                    custom_size: Some(Vec2::new(15.0, 15.0)),
                    ..default()
                },
                Transform::from_translation(transform.translation),
                crate::experience::ExperienceOrb {
                    value: enemy.xp_value,
                },
            ));

            // Despawn health bars
            for (bar_entity, health_bar) in health_bar_query.iter() {
                if health_bar.enemy_entity == entity {
                    commands.entity(bar_entity).despawn();
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

fn update_health_bars(
    enemy_query: Query<(Entity, &Transform, &Enemy, &Sprite)>,
    mut health_bar_bg_query: Query<(&HealthBar, &mut Transform), (With<HealthBarBackground>, Without<Enemy>)>,
    mut health_bar_fg_query: Query<(&HealthBar, &mut Transform, &mut Sprite, &HealthBarForeground), (Without<HealthBarBackground>, Without<Enemy>)>,
) {
    // Update background positions
    for (health_bar, mut bar_transform) in health_bar_bg_query.iter_mut() {
        if let Ok((_, enemy_transform, _, enemy_sprite)) = enemy_query.get(health_bar.enemy_entity) {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y = enemy_transform.translation.y + enemy_size.y / 2.0 + 8.0;
        }
    }

    // Update foreground positions and scale
    for (health_bar, mut bar_transform, mut bar_sprite, bar_fg) in health_bar_fg_query.iter_mut() {
        if let Ok((_, enemy_transform, enemy, enemy_sprite)) = enemy_query.get(health_bar.enemy_entity) {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            let health_percent = (enemy.health / bar_fg.max_health).clamp(0.0, 1.0);

            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y = enemy_transform.translation.y + enemy_size.y / 2.0 + 8.0;

            // Scale the width based on health
            bar_sprite.custom_size = Some(Vec2::new(enemy_size.x * health_percent, 4.0));

            // Offset to align left
            bar_transform.translation.x = enemy_transform.translation.x - (enemy_size.x / 2.0) + (enemy_size.x * health_percent / 2.0);

            // Change color based on health
            if health_percent > 0.5 {
                bar_sprite.color = Color::srgb(0.0, 0.8, 0.0); // Green
            } else if health_percent > 0.25 {
                bar_sprite.color = Color::srgb(0.8, 0.8, 0.0); // Yellow
            } else {
                bar_sprite.color = Color::srgb(0.8, 0.0, 0.0); // Red
            }
        }
    }
}
