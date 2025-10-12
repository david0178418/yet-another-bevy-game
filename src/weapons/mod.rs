use bevy::{asset::AssetLoader, prelude::*};
use crate::behaviors::BehaviorData;
use serde::Deserialize;

mod upgrades;
mod melee;
mod ui;
mod behaviors;

pub use upgrades::{apply_weapon_upgrades, sync_weapon_stats};
pub use melee::{detect_melee_targets, execute_melee_attack, update_melee_hitboxes, update_stunned_enemies};
pub use ui::{spawn_weapon_cooldown_bars, update_weapon_cooldown_bars};
pub use behaviors::{update_orbiting_entities, redistribute_orbiting_entities, update_projectile_spawners, update_despawn_timers, OrbitingEntityCount};

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
	#[serde(default)]
	pub upgrade_behaviors: Vec<crate::behaviors::UpgradeBehavior>,
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
pub struct WeaponInventory {
	pub weapons: std::collections::HashMap<String, (Entity, u32)>, // weapon_id -> (entity, level)
}

#[derive(Component)]
pub struct WeaponName(pub String);

impl Plugin for WeaponsPlugin {
	fn build(&self, app: &mut App) {
		app.init_asset::<WeaponData>()
			.init_asset_loader::<WeaponDataLoader>()
			.init_resource::<OrbitingEntityCount>()
			.init_resource::<WeaponInventory>()
			.add_systems(
				Update,
				(
					initialize_weapon_registry,
					apply_weapon_upgrades,
					sync_weapon_stats,
					redistribute_orbiting_entities,
					update_orbiting_entities,
					update_projectile_spawners,
					update_despawn_timers,
					detect_melee_targets,
					execute_melee_attack,
					update_melee_hitboxes,
					update_stunned_enemies,
					spawn_weapon_cooldown_bars,
					update_weapon_cooldown_bars,
				),
			);
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
	let Some(config_data) = config_assets.get(&config.config_handle) else {
		return;
	};

	let weapons = config_data
		.weapon_ids
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
	weapon_id: &str,
) -> Vec<Entity> {
	use crate::behaviors::*;

	let mut entities = Vec::new();

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
			WeaponName(weapon_data.name.clone()),
			WeaponId(weapon_id.to_string()),
			WeaponLevel(1),
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
				BehaviorData::DamageOnContact {
					damage,
					damage_type,
					targets,
				} => {
					entity_commands.insert((
						DamageOnContact {
							damage: *damage,
							damage_type: *damage_type,
							targets: *targets,
						},
						DamageStats { base: *damage },
					));
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
					let mut timer = Timer::from_seconds(*cooldown, TimerMode::Repeating);
					timer.tick(std::time::Duration::from_secs_f32(*cooldown)); // Start ready to fire
					entity_commands.insert((
						ProjectileSpawner {
							cooldown: timer,
							projectile_template: ProjectileTemplate {
								damage: *damage,
								speed: *speed,
								lifetime: *lifetime,
								size: *projectile_size,
								color: *projectile_color,
							},
							spawn_logic: spawn_logic.clone(),
							fire_range: *fire_range,
						},
						DamageStats { base: *damage },
						CooldownStats { base: *cooldown },
					));
				}
				BehaviorData::MeleeAttack {
					cooldown,
					detection_range,
					damage,
					stun_duration,
					knockback_force,
					attack_duration,
					hitbox_size,
					hitbox_color,
				} => {
					let mut timer = Timer::from_seconds(*cooldown, TimerMode::Repeating);
					timer.tick(std::time::Duration::from_secs_f32(*cooldown)); // Start ready to fire
					entity_commands.insert((
						MeleeAttack {
							cooldown: timer,
							detection_range: *detection_range,
							damage: *damage,
							stun_duration: *stun_duration,
							knockback_force: *knockback_force,
							attack_duration: *attack_duration,
							hitbox_size: *hitbox_size,
							hitbox_color: *hitbox_color,
						},
						DamageStats { base: *damage },
						CooldownStats { base: *cooldown },
						EffectStats {
							base: *stun_duration,
						},
					));
				}
				BehaviorData::FollowPlayer => {
					entity_commands.insert(FollowPlayer);
				}
				BehaviorData::SeekTarget { target_type, speed } => {
					entity_commands.insert(SeekTarget {
						target_type: *target_type,
						speed: *speed,
					});
				}
				BehaviorData::ZigZagMovement {
					base_speed,
					oscillation_speed,
					oscillation_amplitude,
				} => {
					entity_commands.insert(ZigZagMovement {
						base_speed: *base_speed,
						oscillation_speed: *oscillation_speed,
						oscillation_amplitude: *oscillation_amplitude,
						time: 0.0,
					});
				}
				BehaviorData::MaintainDistance {
					target_type,
					preferred_distance,
					speed,
				} => {
					entity_commands.insert(MaintainDistance {
						target_type: *target_type,
						preferred_distance: *preferred_distance,
						speed: *speed,
					});
				}
			}
		}

		// Add UpgradeBehaviors from weapon data
		if !weapon_data.upgrade_behaviors.is_empty() {
			entity_commands.insert(UpgradeBehaviors(weapon_data.upgrade_behaviors.clone()));
		}

		let entity_id = entity_commands.id();
		entities.push(entity_id);
	}

	entities
}
