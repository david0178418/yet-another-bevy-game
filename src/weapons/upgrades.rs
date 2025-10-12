use bevy::prelude::*;

pub type UpgradedWeaponsQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static crate::behaviors::WeaponId,
		&'static mut crate::behaviors::WeaponLevel,
		&'static crate::behaviors::UpgradeBehaviors,
		Option<&'static crate::behaviors::DamageStats>,
		Option<&'static crate::behaviors::CooldownStats>,
		Option<&'static crate::behaviors::EffectStats>,
		Option<&'static mut crate::behaviors::DamageOnContact>,
		Option<&'static mut crate::behaviors::ProjectileSpawner>,
		Option<&'static mut crate::behaviors::MeleeAttack>,
	),
	Changed<crate::behaviors::WeaponLevel>,
>;

// Generic system that applies upgrades to weapons based on their UpgradeBehaviors
pub fn apply_weapon_upgrades(
	mut commands: Commands,
	mut upgraded_weapons: UpgradedWeaponsQuery,
	weapon_registry: Option<Res<super::WeaponRegistry>>,
	weapon_data_assets: Res<Assets<super::WeaponData>>,
	weapon_inventory: Option<Res<super::WeaponInventory>>,
) {
	for (
		entity,
		weapon_id,
		weapon_level,
		upgrade_behaviors,
		damage_stats,
		cooldown_stats,
		effect_stats,
		mut damage_on_contact,
		mut projectile,
		mut melee,
	) in upgraded_weapons.iter_mut()
	{
		// Check if this entity is the primary weapon in the inventory
		let is_primary = weapon_inventory
			.as_ref()
			.and_then(|inv| inv.weapons.get(&weapon_id.0))
			.map(|(primary_entity, _)| *primary_entity == entity)
			.unwrap_or(false);
		for behavior in &upgrade_behaviors.0 {
			match behavior {
				crate::behaviors::UpgradeBehavior::ScaleDamage { per_level } => {
					if let Some(damage_stats) = damage_stats {
						let multiplier = 1.0 + (weapon_level.0 as f32 - 1.0) * per_level;
						let new_damage = damage_stats.base * multiplier;

						// Apply to DamageOnContact if present
						if let Some(ref mut contact) = damage_on_contact {
							contact.damage = new_damage;
						}

						// Apply to ProjectileSpawner if present
						if let Some(ref mut proj) = projectile {
							proj.projectile_template.damage = new_damage;
						}

						// Apply to MeleeAttack if present
						if let Some(ref mut mel) = melee {
							mel.damage = new_damage;
						}
					}
				}
				crate::behaviors::UpgradeBehavior::ReduceCooldown {
					per_level,
					min_multiplier,
				} => {
					if let Some(cooldown_stats) = cooldown_stats {
						let multiplier =
							(1.0 - (weapon_level.0 as f32 - 1.0) * per_level).max(*min_multiplier);
						let new_cooldown = cooldown_stats.base * multiplier;
						let duration = std::time::Duration::from_secs_f32(new_cooldown);

						if let Some(ref mut proj) = projectile {
							proj.cooldown.set_duration(duration);
						}
						if let Some(ref mut mel) = melee {
							mel.cooldown.set_duration(duration);
						}
					}
				}
				crate::behaviors::UpgradeBehavior::IncreaseEffect { per_level } => {
					if let Some(effect_stats) = effect_stats {
						let multiplier = 1.0 + (weapon_level.0 as f32 - 1.0) * per_level;
						let new_effect = effect_stats.base * multiplier;

						if let Some(ref mut mel) = melee {
							mel.stun_duration = new_effect;
						}
					}
				}
				crate::behaviors::UpgradeBehavior::SpawnAdditionalEntity => {
					// Only spawn additional entities for the primary weapon in inventory
					// This prevents cascade spawning when newly spawned entities get their level set
					if !is_primary {
						continue;
					}

					// Spawn one additional entity (e.g., for orbiting blades)
					if let Some(registry) = &weapon_registry {
						if let Some(weapon_handle) = registry.get(&weapon_id.0) {
							if let Some(weapon_data) = weapon_data_assets.get(weapon_handle) {
								let new_entities = super::spawn_entity_from_data(
									&mut commands,
									weapon_data,
									1,
									&weapon_id.0,
								);

								// Set new entities to the current weapon level
								for new_entity in new_entities {
									commands.entity(new_entity).insert(*weapon_level);
								}
							}
						}
					}
				}
			}
		}
	}
}

// Sync damage across all entities with the same WeaponId (for weapons with multiple instances like orbiting blades)
pub fn sync_weapon_stats(
	mut weapon_entities: Query<(
		&crate::behaviors::WeaponId,
		&crate::behaviors::WeaponLevel,
		&crate::behaviors::DamageStats,
		&mut crate::behaviors::DamageOnContact,
	)>,
) {
	use std::collections::HashMap;

	// Find the highest level and damage for each weapon_id
	let mut weapon_stats: HashMap<String, (u32, f32)> = HashMap::new();

	for (weapon_id, level, damage_stats, _) in weapon_entities.iter() {
		let entry = weapon_stats.entry(weapon_id.0.clone()).or_insert((0, 0.0));
		if level.0 > entry.0 {
			entry.0 = level.0;
			entry.1 = damage_stats.base;
		}
	}

	// Update all entities with the same weapon_id to have matching damage
	for (weapon_id, _, damage_stats, mut contact) in weapon_entities.iter_mut() {
		if let Some((max_level, _)) = weapon_stats.get(&weapon_id.0) {
			// Recalculate damage based on max level
			// This assumes ScaleDamage behavior - we could make this more sophisticated
			let multiplier = 1.0
				+ (*max_level as f32 - 1.0) * crate::constants::WEAPON_DAMAGE_INCREASE_PER_LEVEL;
			contact.damage = damage_stats.base * multiplier;
		}
	}
}
