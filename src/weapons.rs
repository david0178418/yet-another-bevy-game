use bevy::{prelude::*, asset::AssetLoader};
use std::f32::consts::PI;
use serde::Deserialize;
use crate::behaviors::BehaviorData;

pub struct WeaponsPlugin;

// Visual data for weapons
#[derive(Deserialize, Clone)]
pub struct VisualData {
	pub size: (f32, f32),
	pub color: (f32, f32, f32),
}

// Generic weapon data structure
#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct WeaponData {
	pub name: String,
	pub description: String,
	pub visual: VisualData,
	pub behaviors: Vec<BehaviorData>,
}

#[derive(Default)]
struct WeaponDataLoader;

impl AssetLoader for WeaponDataLoader {
	type Asset = WeaponData;
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
		let data = ron::de::from_bytes::<WeaponData>(&bytes)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		Ok(data)
	}

	fn extensions(&self) -> &[&str] {
		&["weapon.ron"]
	}
}

#[derive(Resource)]
pub struct WeaponRegistry {
	weapons: std::collections::HashMap<String, Handle<WeaponData>>,
}

impl WeaponRegistry {
	pub fn get(&self, id: &str) -> Option<&Handle<WeaponData>> {
		self.weapons.get(id)
	}
}

#[derive(Resource, Default)]
struct OrbitingEntityCount(usize);

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<WeaponData>()
            .init_asset_loader::<WeaponDataLoader>()
            .init_resource::<OrbitingEntityCount>()
            .add_systems(Update, (
                initialize_weapon_registry,
                redistribute_orbiting_entities,
                update_orbiting_entities,
                update_projectile_spawners,
                update_despawn_timers,
            ));
    }
}

fn initialize_weapon_registry(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	registry: Option<Res<WeaponRegistry>>,
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

	let weapons = config_data.weapon_ids
		.iter()
		.map(|id| {
			let path = format!("weapons/{}.weapon.ron", id);
			(id.clone(), asset_server.load(path))
		})
		.collect();

	commands.insert_resource(WeaponRegistry { weapons });
}

// Generic spawn function that creates entities from weapon data
pub fn spawn_entity_from_data(
    commands: &mut Commands,
    weapon_data: &WeaponData,
    count: u32,
) {
	use crate::behaviors::*;

	for _ in 0..count {
		let mut entity_commands = commands.spawn((
			Sprite {
				color: Color::srgb(
					weapon_data.visual.color.0,
					weapon_data.visual.color.1,
					weapon_data.visual.color.2,
				),
				custom_size: Some(Vec2::new(
					weapon_data.visual.size.0,
					weapon_data.visual.size.1,
				)),
				..default()
			},
			Transform::from_xyz(0.0, 0.0, 1.0),
		));

		// Add components based on behaviors
		for behavior in &weapon_data.behaviors {
			match behavior {
				BehaviorData::Orbiting { radius, speed } => {
					entity_commands.insert(OrbitingBehavior {
						radius: *radius,
						speed: *speed,
						angle: 0.0, // Will be set by redistribution
					});
				}
				BehaviorData::DamageOnContact { damage, damage_type, targets } => {
					entity_commands.insert(DamageOnContact {
						damage: *damage,
						damage_type: *damage_type,
						targets: *targets,
					});
				}
				BehaviorData::ProjectileSpawner {
					cooldown,
					damage,
					speed,
					lifetime,
					projectile_size,
					projectile_color,
					spawn_logic,
					fire_range,
				} => {
					entity_commands.insert(ProjectileSpawner {
						cooldown: Timer::from_seconds(*cooldown, TimerMode::Repeating),
						projectile_template: ProjectileTemplate {
							damage: *damage,
							speed: *speed,
							lifetime: *lifetime,
							size: *projectile_size,
							color: *projectile_color,
						},
						spawn_logic: spawn_logic.clone(),
						fire_range: *fire_range,
					});
				}
				BehaviorData::FollowPlayer => {
					entity_commands.insert(FollowPlayer);
				}
			}
		}
	}
}

// Generic update system for orbiting entities
fn update_orbiting_entities(
    mut orbiting_query: Query<(&mut Transform, &mut crate::behaviors::OrbitingBehavior, &crate::behaviors::FollowPlayer)>,
    player_query: Query<&Transform, (With<crate::behaviors::PlayerTag>, Without<crate::behaviors::OrbitingBehavior>)>,
    time: Res<Time<Virtual>>,
) {
    if let Ok(player_transform) = player_query.single() {
		for (mut transform, mut behavior, _) in orbiting_query.iter_mut() {
			// Rotate based on speed
			behavior.angle += behavior.speed * time.delta_secs();

			// Update position relative to player
			transform.translation.x = player_transform.translation.x + behavior.angle.cos() * behavior.radius;
			transform.translation.y = player_transform.translation.y + behavior.angle.sin() * behavior.radius;
			transform.rotation = Quat::from_rotation_z(behavior.angle + PI / 2.0);
		}
    }
}

// System to redistribute orbiting entities when new ones are added
fn redistribute_orbiting_entities(
	mut all_orbiting: Query<&mut crate::behaviors::OrbitingBehavior, With<crate::behaviors::FollowPlayer>>,
	mut count_tracker: ResMut<OrbitingEntityCount>,
) {
	let current_count = all_orbiting.iter().count();

	// Only redistribute if count changed (new entities added or removed)
	if current_count != count_tracker.0 {
		count_tracker.0 = current_count;

		// Redistribute all entities evenly
		for (index, mut behavior) in all_orbiting.iter_mut().enumerate() {
			behavior.angle = (index as f32 / current_count as f32) * 2.0 * PI;
		}
	}
}

// Generic update system for projectile spawners
fn update_projectile_spawners(
    mut commands: Commands,
    mut spawner_query: Query<(&mut crate::behaviors::ProjectileSpawner, &crate::behaviors::FollowPlayer)>,
    player_query: Query<&Transform, With<crate::behaviors::PlayerTag>>,
    enemy_query: Query<&Transform, With<crate::behaviors::EnemyTag>>,
    time: Res<Time<Virtual>>,
) {
    use crate::behaviors::*;

    if let Ok(player_transform) = player_query.single() {
        for (mut spawner, _) in spawner_query.iter_mut() {
            if spawner.cooldown.tick(time.delta()).just_finished() {
                let spawn_direction = match &spawner.spawn_logic {
                    SpawnLogic::NearestEnemy => {
                        // Find nearest enemy (optionally within range)
                        let nearest_enemy = enemy_query.iter()
                            .filter(|enemy_transform| {
                                // If fire_range is set, only consider enemies within range
                                if let Some(range) = spawner.fire_range {
                                    player_transform.translation.distance(enemy_transform.translation) <= range
                                } else {
                                    true  // No range limit
                                }
                            })
                            .min_by(|a, b| {
                                let dist_a = player_transform.translation.distance(a.translation);
                                let dist_b = player_transform.translation.distance(b.translation);
                                dist_a.partial_cmp(&dist_b).unwrap()
                            });

                        // If no enemy in range, don't fire
                        if let Some(enemy_transform) = nearest_enemy {
                            let direction = Vec2::new(
                                enemy_transform.translation.x - player_transform.translation.x,
                                enemy_transform.translation.y - player_transform.translation.y,
                            );
                            Some(direction.normalize())
                        } else {
                            // No enemy in range, skip spawning projectile
                            None
                        }
                    }
                    SpawnLogic::PlayerDirection => Some(Vec2::new(1.0, 0.0)), // Could be enhanced with actual player direction
                    SpawnLogic::Fixed(x, y) => {
                        let direction = Vec2::new(*x, *y);
                        if direction.length_squared() > 0.0 {
                            Some(direction.normalize())
                        } else {
                            Some(Vec2::new(1.0, 0.0))
                        }
                    }
                };

                let Some(direction) = spawn_direction else {
                    continue;
                };

                // Spawn projectile
                let template = &spawner.projectile_template;
                let angle = direction.y.atan2(direction.x);
                commands.spawn((
                    Sprite {
                        color: Color::srgb(template.color.0, template.color.1, template.color.2),
                        custom_size: Some(Vec2::new(template.size.0, template.size.1)),
                        ..default()
                    },
                    Transform::from_xyz(
                        player_transform.translation.x + direction.x * 30.0,
                        player_transform.translation.y + direction.y * 30.0,
                        0.0,
                    ).with_rotation(Quat::from_rotation_z(angle)),
                    crate::physics::Velocity {
                        x: direction.x * template.speed,
                        y: direction.y * template.speed,
                    },
                    DamageOnContact {
                        damage: template.damage,
                        damage_type: DamageType::OneTime,
                        targets: TargetFilter::Enemies,
                    },
                    DespawnOnTimer {
                        timer: Timer::from_seconds(template.lifetime, TimerMode::Once),
                    },
                    ProjectileTag,
                ));
            }
        }
    }
}

// Generic despawn timer system
fn update_despawn_timers(
    mut commands: Commands,
    mut query: Query<(Entity, &mut crate::behaviors::DespawnOnTimer)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut despawn_timer) in query.iter_mut() {
        if despawn_timer.timer.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
