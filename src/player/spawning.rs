use bevy::prelude::*;

#[derive(Component)]
pub struct NeedsInitialWeapons {
	pub weapons: Vec<crate::InitialWeapon>,
}

fn spawn_platform(commands: &mut Commands, position: Vec3, size: Vec2) {
	commands.spawn((
		Sprite {
			color: crate::constants::PLATFORM_COLOR,
			custom_size: Some(size),
			..default()
		},
		Transform::from_translation(position),
		crate::physics::Ground,
		crate::physics::Collider,
	));
}

pub fn spawn_platforms(commands: &mut Commands) {
	const GROUND_SIZE: Vec2 = Vec2::new(2000.0, 40.0);
	const STAIR_SIZE: Vec2 = Vec2::new(150.0, 20.0);
	const TOP_PLATFORM_SIZE: Vec2 = Vec2::new(200.0, 20.0);

	const PLATFORMS: [(Vec3, Vec2); 8] = [
		(Vec3::new(0.0, -300.0, 0.0), GROUND_SIZE),
		(Vec3::new(-200.0, -240.0, 0.0), STAIR_SIZE),
		(Vec3::new(-400.0, -180.0, 0.0), STAIR_SIZE),
		(Vec3::new(-200.0, -120.0, 0.0), STAIR_SIZE),
		(Vec3::new(200.0, -240.0, 0.0), STAIR_SIZE),
		(Vec3::new(400.0, -180.0, 0.0), STAIR_SIZE),
		(Vec3::new(200.0, -120.0, 0.0), STAIR_SIZE),
		(Vec3::new(0.0, -60.0, 0.0), TOP_PLATFORM_SIZE),
	];

	PLATFORMS
		.iter()
		.for_each(|(pos, size)| spawn_platform(commands, *pos, *size));
}

pub fn spawn_player(
	mut commands: Commands,
	game_config: Option<Res<crate::GameConfig>>,
	config_assets: Res<Assets<crate::GameConfigData>>,
	player_query: Query<(), With<super::Player>>,
	ui_query: Query<(), With<super::ui::PlayerStatsText>>,
	platform_query: Query<(), With<crate::physics::Ground>>,
) {
	// Only spawn once
	if !player_query.is_empty() {
		return;
	}

	let Some(game_config) = game_config else {
		return;
	};

	let Some(config_data) = config_assets.get(&game_config.config_handle) else {
		return;
	};

	commands.spawn((
		Sprite {
			color: crate::constants::PLAYER_COLOR,
			custom_size: Some(crate::constants::PLAYER_SIZE),
			..default()
		},
		Transform::from_translation(crate::constants::PLAYER_SPAWN_POSITION),
		super::Player::default(),
		crate::behaviors::PlayerTag,
		crate::behaviors::Damageable {
			health: crate::constants::PLAYER_DEFAULT_HEALTH,
			max_health: crate::constants::PLAYER_DEFAULT_HEALTH,
		},
		crate::behaviors::PlayerEnergy {
			current: crate::constants::PLAYER_DEFAULT_ENERGY,
			max: crate::constants::PLAYER_DEFAULT_ENERGY,
			regen_rate: crate::constants::PLAYER_ENERGY_REGEN_RATE,
			repulsion_force: crate::constants::REPULSION_FORCE_DEFAULT,
		},
		crate::physics::Velocity { x: 0.0, y: 0.0 },
		crate::physics::Grounded(false),
		crate::physics::Collider,
		NeedsInitialWeapons {
			weapons: config_data.initial_weapons.clone(),
		},
	));

	// Only spawn platforms if they don't exist
	if platform_query.is_empty() {
		spawn_platforms(&mut commands);
	}

	// Only spawn UI if it doesn't exist
	if ui_query.is_empty() {
		super::ui::spawn_player_ui(&mut commands);
	}
}

pub fn spawn_initial_weapon(
	mut commands: Commands,
	mut player_query: Query<(Entity, &NeedsInitialWeapons)>,
	weapon_registry: Option<Res<crate::weapons::WeaponRegistry>>,
	weapon_data_assets: Res<Assets<crate::weapons::WeaponData>>,
	mut weapon_inventory: Option<ResMut<crate::weapons::WeaponInventory>>,
) {
	let Some(registry) = weapon_registry else {
		return;
	};

	for (entity, needs_weapons) in player_query.iter_mut() {
		// Spawn each initial weapon
		for weapon_config in &needs_weapons.weapons {
			if let Some(handle) = registry.get(&weapon_config.weapon_id) {
				if let Some(weapon_data) = weapon_data_assets.get(handle) {
					let weapon_entities = crate::weapons::spawn_entity_from_data(
						&mut commands,
						weapon_data,
						weapon_config.level,
						&weapon_config.weapon_id,
					);

					// Add to inventory
					if let Some(inventory) = &mut weapon_inventory {
						if !weapon_entities.is_empty() {
							inventory.weapons.insert(
								weapon_config.weapon_id.clone(),
								(weapon_entities[0], weapon_config.level),
							);
						}
					}
				}
			}
		}

		// Remove the component after spawning all weapons
		commands.entity(entity).remove::<NeedsInitialWeapons>();
	}
}
