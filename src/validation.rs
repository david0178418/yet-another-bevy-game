use bevy::prelude::*;
use std::collections::HashSet;

pub struct ValidationPlugin;

impl Plugin for ValidationPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<ValidationState>()
			.add_systems(Update, validate_game_config);
	}
}

#[derive(Resource, Default)]
struct ValidationState {
	validated: bool,
}

fn validate_game_config(
	game_config: Option<Res<crate::GameConfig>>,
	config_assets: Res<Assets<crate::GameConfigData>>,
	weapon_registry: Option<Res<crate::weapons::WeaponRegistry>>,
	enemy_registry: Option<Res<crate::enemy::EnemyRegistry>>,
	weapon_assets: Res<Assets<crate::weapons::WeaponData>>,
	enemy_assets: Res<Assets<crate::enemy::EnemyData>>,
	mut validation_state: ResMut<ValidationState>,
) {
	if validation_state.validated {
		return;
	}

	// Wait for registries to be initialized
	let (Some(config), Some(_weapon_registry), Some(_enemy_registry)) =
		(game_config, weapon_registry, enemy_registry)
	else {
		return;
	};

	let Some(config_data) = config_assets.get(&config.config_handle) else {
		return;
	};

	let mut errors = Vec::new();

	// Validate weapon IDs
	validate_ids(&config_data.weapon_ids, "weapon", &mut errors);

	// Validate enemy IDs
	validate_ids(&config_data.enemy_ids, "enemy", &mut errors);

	// Validate initial weapons reference valid weapon IDs
	validate_initial_weapons(config_data, &mut errors);

	// Validate powerup pool references
	validate_powerup_pool(config_data, &mut errors);

	// Validate asset loading status
	validate_asset_loading(config_data, &weapon_assets, &enemy_assets, &mut errors);

	if !errors.is_empty() {
		error!("Asset validation failed with {} error(s):", errors.len());
		for (i, err) in errors.iter().enumerate() {
			error!("  {}. {}", i + 1, err);
		}
		panic!("Asset validation failed. Please fix the errors above.");
	}

	info!("Asset validation passed successfully");
	validation_state.validated = true;
}

fn validate_ids(ids: &[String], id_type: &str, errors: &mut Vec<String>) {
	let mut seen = HashSet::new();

	for id in ids {
		// Check for empty IDs
		if id.is_empty() {
			errors.push(format!(
				"Empty {} ID found in game_config.ron",
				id_type
			));
			continue;
		}

		// Check for duplicate IDs
		if !seen.insert(id) {
			errors.push(format!(
				"Duplicate {} ID '{}' found in game_config.ron",
				id_type, id
			));
		}

		// Check for invalid characters (optional, but helpful)
		if id.contains(char::is_whitespace) {
			errors.push(format!(
				"Invalid {} ID '{}' contains whitespace",
				id_type, id
			));
		}
	}
}

fn validate_initial_weapons(
	config_data: &crate::GameConfigData,
	errors: &mut Vec<String>,
) {
	let valid_weapon_ids: HashSet<_> = config_data.weapon_ids.iter().collect();

	for initial_weapon in &config_data.initial_weapons {
		if !valid_weapon_ids.contains(&initial_weapon.weapon_id) {
			errors.push(format!(
				"Initial weapon references unknown weapon ID '{}'",
				initial_weapon.weapon_id
			));
		}

		if initial_weapon.level == 0 {
			errors.push(format!(
				"Initial weapon '{}' has invalid level 0 (levels start at 1)",
				initial_weapon.weapon_id
			));
		}
	}
}

fn validate_powerup_pool(
	config_data: &crate::GameConfigData,
	errors: &mut Vec<String>,
) {
	let valid_weapon_ids: HashSet<_> = config_data.weapon_ids.iter().collect();

	for powerup in &config_data.powerup_pool {
		if let crate::PowerupDefinition::Weapon(weapon_id) = powerup {
			if !valid_weapon_ids.contains(weapon_id) {
				errors.push(format!(
					"Powerup pool references unknown weapon ID '{}'",
					weapon_id
				));
			}
		}
	}
}

fn validate_asset_loading(
	config_data: &crate::GameConfigData,
	weapon_assets: &Assets<crate::weapons::WeaponData>,
	enemy_assets: &Assets<crate::enemy::EnemyData>,
	errors: &mut Vec<String>,
) {
	// Check if all weapon assets are loaded
	for weapon_id in &config_data.weapon_ids {
		if weapon_id.is_empty() {
			continue; // Already reported
		}

		// We can't directly check the asset handle here without the registry,
		// but we can check if the asset path would be valid
		if weapon_id.contains("..") || weapon_id.contains('/') {
			errors.push(format!(
				"Weapon ID '{}' contains invalid path characters",
				weapon_id
			));
		}
	}

	// Check if all enemy assets are loaded
	for enemy_id in &config_data.enemy_ids {
		if enemy_id.is_empty() {
			continue; // Already reported
		}

		if enemy_id.contains("..") || enemy_id.contains('/') {
			errors.push(format!(
				"Enemy ID '{}' contains invalid path characters",
				enemy_id
			));
		}
	}

	// Log warnings for assets that haven't loaded yet
	let weapon_count = weapon_assets.len();
	let enemy_count = enemy_assets.len();

	if weapon_count < config_data.weapon_ids.len() {
		warn!(
			"Only {}/{} weapon assets have loaded",
			weapon_count,
			config_data.weapon_ids.len()
		);
	}

	if enemy_count < config_data.enemy_ids.len() {
		warn!(
			"Only {}/{} enemy assets have loaded",
			enemy_count,
			config_data.enemy_ids.len()
		);
	}
}
