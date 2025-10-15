use bevy::prelude::*;

pub fn apply_powerup(
	powerup_def: &crate::PowerupDefinition,
	commands: &mut Commands,
	player_stats: (
		&mut crate::player::Player,
		&mut crate::behaviors::Damageable,
		&mut crate::behaviors::PlayerEnergy,
	),
	weapon_resources: &super::WeaponResources,
	weapon_inventory: &mut crate::weapons::WeaponInventory,
	weapon_level_query: &mut Query<&mut crate::behaviors::WeaponLevel>,
) {
	let (player, player_damageable, player_energy) = player_stats;
	match powerup_def {
		crate::PowerupDefinition::Weapon(weapon_id) => {
			// Check if player already owns this weapon
			if let Some((entity, _current_level)) = weapon_inventory.weapons.get(weapon_id) {
				// Upgrade existing weapon by incrementing its level
				// The generic upgrade system will handle the rest
				let entity = *entity;
				if let Ok(mut level) = weapon_level_query.get_mut(entity) {
					level.0 += 1;
					weapon_inventory
						.weapons
						.insert(weapon_id.clone(), (entity, level.0));
				}
			} else {
				// Spawn new weapon
				if let Some(registry) = weapon_resources.registry.as_ref() {
					if let Some(handle) = registry.get(weapon_id) {
						if let Some(weapon_data) = weapon_resources.assets.get(handle) {
							let entities = crate::weapons::spawn_entity_from_data(
								commands,
								weapon_data,
								1,
								weapon_id,
							);
							if !entities.is_empty() {
								weapon_inventory
									.weapons
									.insert(weapon_id.clone(), (entities[0], 1));
							}
						}
					}
				}
			}
		}
		crate::PowerupDefinition::StatBoost(boost) => match boost.stat {
			crate::StatType::Speed => {
				player.speed += boost.value;
			}
			crate::StatType::JumpForce => {
				player.jump_force += boost.value;
			}
			crate::StatType::MaxHealth => {
				player_damageable.max_health += boost.value;
				player_damageable.health = player_damageable.max_health;
			}
			crate::StatType::EnergyRegen => {
				player_energy.regen_rate += boost.value;
			}
			crate::StatType::RepulsionForce => {
				player_energy.repulsion_force += boost.value;
			}
		},
	}
}
