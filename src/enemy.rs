use bevy::{prelude::*, asset::AssetLoader};
use rand::Rng;
use serde::Deserialize;

pub struct EnemyPlugin;

type HealthBarBgQuery<'w, 's> = Query<'w, 's, (&'static HealthBar, &'static mut Transform), (With<HealthBarBackground>, Without<crate::behaviors::EnemyTag>)>;
type HealthBarFgQuery<'w, 's> = Query<'w, 's, (&'static HealthBar, &'static mut Transform, &'static mut Sprite, &'static HealthBarForeground), (Without<HealthBarBackground>, Without<crate::behaviors::EnemyTag>)>;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<EnemyData>()
            .init_asset_loader::<EnemyDataLoader>()
            .insert_resource(EnemySpawnTimer(Timer::from_seconds(crate::constants::ENEMY_SPAWN_TIMER, TimerMode::Repeating)))
            .insert_resource(WaveTimer {
                timer: Timer::from_seconds(crate::constants::WAVE_DURATION, TimerMode::Repeating),
                wave: 1,
            })
            .add_systems(Update, (
                initialize_enemy_registry,
                spawn_enemies,
                move_enemies,
                update_wave,
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
    pub speed: f32,
    pub xp_value: u32,
}

#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct EnemyData {
	pub color: (f32, f32, f32),
	pub base_health: f32,
	pub speed: f32,
	pub damage: f32,
	pub size: (f32, f32),
	pub xp_value: u32,
}

#[derive(Default)]
struct EnemyDataLoader;

impl AssetLoader for EnemyDataLoader {
	type Asset = EnemyData;
	type Settings = ();
	type Error = std::io::Error;

	async fn load(
		&self,
		reader: &mut dyn bevy::asset::io::Reader,
		_settings: &Self::Settings,
		_load_context: &mut bevy::asset::LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).await?;
		let data = ron::de::from_bytes::<EnemyData>(&bytes)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		Ok(data)
	}

	fn extensions(&self) -> &[&str] {
		&["enemy.ron"]
	}
}

#[derive(Resource)]
pub struct EnemyRegistry {
	enemies: std::collections::HashMap<String, Handle<EnemyData>>,
	enemy_ids: Vec<String>,
}

impl EnemyRegistry {
	pub fn get(&self, id: &str) -> Option<&Handle<EnemyData>> {
		self.enemies.get(id)
	}

	pub fn random_id(&self) -> Option<&str> {
		if self.enemy_ids.is_empty() {
			return None;
		}
		let mut rng = rand::thread_rng();
		let index = rng.gen_range(0..self.enemy_ids.len());
		Some(&self.enemy_ids[index])
	}
}


fn initialize_enemy_registry(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	registry: Option<Res<EnemyRegistry>>,
	game_config: Option<Res<crate::GameConfig>>,
	config_assets: Res<Assets<crate::GameConfigData>>,
) {
	// Only initialize once
	if registry.is_some() {
		return;
	}

	// Wait for game config to load
	let Some(config) = game_config else { return };
	let Some(config_data) = config_assets.get(&config.config_handle) else { return };

	let enemies = config_data.enemy_ids
		.iter()
		.map(|id| {
			let path = format!("enemies/{}.enemy.ron", id);
			(id.clone(), asset_server.load(path))
		})
		.collect();

	commands.insert_resource(EnemyRegistry {
		enemies,
		enemy_ids: config_data.enemy_ids.clone(),
	});
}

#[derive(Component)]
pub struct HealthBar {
    pub enemy_entity: Entity,
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
    enemy_registry: Option<Res<EnemyRegistry>>,
    enemy_data_assets: Res<Assets<EnemyData>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut rng = rand::thread_rng();

        if let (Ok(player_transform), Some(registry)) = (player_query.single(), enemy_registry) {
            // Spawn enemies off-screen
            let spawn_side = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
            let spawn_x = player_transform.translation.x + spawn_side * crate::constants::ENEMY_SPAWN_DISTANCE;
            let spawn_y = rng.gen_range(crate::constants::ENEMY_SPAWN_Y_MIN..crate::constants::ENEMY_SPAWN_Y_MAX);

            let Some(enemy_id) = registry.random_id() else { return };
            let Some(enemy_handle) = registry.get(enemy_id) else { return };

            // Wait for asset to be loaded
            let Some(enemy_data) = enemy_data_assets.get(enemy_handle) else {
                return;
            };

            let size = Vec2::new(enemy_data.size.0, enemy_data.size.1);
            let scaled_health = enemy_data.base_health * (1.0 + (wave.wave as f32 * crate::constants::WAVE_HEALTH_SCALING));

            let enemy_entity = commands.spawn((
                Sprite {
                    color: Color::srgb(enemy_data.color.0, enemy_data.color.1, enemy_data.color.2),
                    custom_size: Some(size),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y, 0.0),
                Enemy {
                    speed: enemy_data.speed,
                    xp_value: enemy_data.xp_value,
                },
                crate::behaviors::Damageable {
                    health: scaled_health,
                    max_health: scaled_health,
                },
                crate::behaviors::DamageOnContact {
                    damage: enemy_data.damage,
                    damage_type: crate::behaviors::DamageType::Continuous,
                    targets: crate::behaviors::TargetFilter::Player,
                },
                crate::behaviors::EnemyTag,
                crate::physics::Velocity { x: 0.0, y: 0.0 },
                crate::physics::Grounded(false),
                crate::physics::Collider,
            )).id();

            // Spawn health bar background
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.2, 0.2, 0.2),
                    custom_size: Some(Vec2::new(size.x, crate::constants::HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y + size.y / 2.0 + crate::constants::HEALTH_BAR_OFFSET_Y, 1.0),
                HealthBar { enemy_entity },
                HealthBarBackground,
            ));

            // Spawn health bar foreground
            commands.spawn((
                Sprite {
                    color: Color::srgb(0.0, 0.8, 0.0),
                    custom_size: Some(Vec2::new(size.x, crate::constants::HEALTH_BAR_HEIGHT)),
                    ..default()
                },
                Transform::from_xyz(spawn_x, spawn_y + size.y / 2.0 + crate::constants::HEALTH_BAR_OFFSET_Y, 2.0),
                HealthBar { enemy_entity },
                HealthBarForeground { max_health: scaled_health },
            ));
        }
    }
}

fn move_enemies(
    mut enemy_query: Query<(&Transform, &mut crate::physics::Velocity, &Enemy, Has<crate::behaviors::Stunned>), Without<crate::player::Player>>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (enemy_transform, mut velocity, enemy, is_stunned) in enemy_query.iter_mut() {
            // Skip stunned enemies - they shouldn't move toward player
            if is_stunned {
                continue;
            }

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
        let new_duration = (crate::constants::ENEMY_SPAWN_TIMER - (wave.wave as f32 * crate::constants::WAVE_SPAWN_RATE_SCALING)).max(crate::constants::MIN_SPAWN_DURATION);
        spawn_timer.0.set_duration(std::time::Duration::from_secs_f32(new_duration));
    }
}

fn update_health_bars(
    enemy_query: Query<(Entity, &Transform, &crate::behaviors::Damageable, &Sprite), With<crate::behaviors::EnemyTag>>,
    mut health_bar_bg_query: HealthBarBgQuery,
    mut health_bar_fg_query: HealthBarFgQuery,
) {
    // Update background positions
    for (health_bar, mut bar_transform) in health_bar_bg_query.iter_mut() {
        if let Ok((_, enemy_transform, _, enemy_sprite)) = enemy_query.get(health_bar.enemy_entity) {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y = enemy_transform.translation.y + enemy_size.y / 2.0 + crate::constants::HEALTH_BAR_OFFSET_Y;
        }
    }

    // Update foreground positions and scale
    for (health_bar, mut bar_transform, mut bar_sprite, bar_fg) in health_bar_fg_query.iter_mut() {
        if let Ok((_, enemy_transform, damageable, enemy_sprite)) = enemy_query.get(health_bar.enemy_entity) {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);
            let health_percent = (damageable.health / bar_fg.max_health).clamp(0.0, 1.0);

            bar_transform.translation.x = enemy_transform.translation.x;
            bar_transform.translation.y = enemy_transform.translation.y + enemy_size.y / 2.0 + crate::constants::HEALTH_BAR_OFFSET_Y;

            // Scale the width based on health
            bar_sprite.custom_size = Some(Vec2::new(enemy_size.x * health_percent, crate::constants::HEALTH_BAR_HEIGHT));

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
