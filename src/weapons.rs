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

#[derive(Resource, Default)]
struct OrbitingEntityCount(usize);

// UI Components for weapon cooldowns
#[derive(Component)]
struct WeaponCooldownBar {
	weapon_entity: Entity,
}

#[derive(Component)]
struct WeaponCooldownBarBackground;

#[derive(Component)]
struct WeaponCooldownBarForeground;

#[derive(Component)]
struct WeaponCooldownText;

#[derive(Component)]
struct HasCooldownUI;

#[derive(Component)]
struct WeaponName(String);

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<WeaponData>()
            .init_asset_loader::<WeaponDataLoader>()
            .init_resource::<OrbitingEntityCount>()
            .init_resource::<WeaponInventory>()
            .add_systems(Update, (
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

// ============ Generic Upgrade System ============

// Generic system that applies upgrades to weapons based on their UpgradeBehaviors
fn apply_weapon_upgrades(
	mut commands: Commands,
	mut upgraded_weapons: Query<
		(
			Entity,
			&crate::behaviors::WeaponId,
			&mut crate::behaviors::WeaponLevel,
			&crate::behaviors::UpgradeBehaviors,
			Option<&crate::behaviors::DamageStats>,
			Option<&crate::behaviors::CooldownStats>,
			Option<&crate::behaviors::EffectStats>,
			Option<&mut crate::behaviors::DamageOnContact>,
			Option<&mut crate::behaviors::ProjectileSpawner>,
			Option<&mut crate::behaviors::MeleeAttack>,
		),
		Changed<crate::behaviors::WeaponLevel>,
	>,
	weapon_registry: Option<Res<WeaponRegistry>>,
	weapon_data_assets: Res<Assets<WeaponData>>,
	weapon_inventory: Option<Res<WeaponInventory>>,
) {
	for (entity, weapon_id, weapon_level, upgrade_behaviors, damage_stats, cooldown_stats, effect_stats,
	     mut damage_on_contact, mut projectile, mut melee) in upgraded_weapons.iter_mut() {
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
				crate::behaviors::UpgradeBehavior::ReduceCooldown { per_level, min_multiplier } => {
					if let Some(cooldown_stats) = cooldown_stats {
						let multiplier = (1.0 - (weapon_level.0 as f32 - 1.0) * per_level).max(*min_multiplier);
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
								let new_entities = spawn_entity_from_data(&mut commands, weapon_data, 1, &weapon_id.0);

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
fn sync_weapon_stats(
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
			let multiplier = 1.0 + (*max_level as f32 - 1.0) * crate::constants::WEAPON_DAMAGE_INCREASE_PER_LEVEL;
			contact.damage = damage_stats.base * multiplier;
		}
	}
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
				BehaviorData::DamageOnContact { damage, damage_type, targets } => {
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
						EffectStats { base: *stun_duration },
					));
				}
				BehaviorData::FollowPlayer => {
					entity_commands.insert(FollowPlayer);
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
            // Only tick if not finished (actively cooling down)
            if !spawner.cooldown.is_finished() {
                spawner.cooldown.tick(time.delta());
                continue; // Skip to next weapon while cooling down
            }

            // Cooldown is ready, try to fire
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

            // Reset cooldown after firing
            spawner.cooldown.reset();

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

// ============ Melee Attack Systems ============

fn detect_melee_targets(
    mut commands: Commands,
    mut melee_query: Query<&mut crate::behaviors::MeleeAttack, With<crate::behaviors::FollowPlayer>>,
    player_query: Query<(Entity, &Transform), With<crate::behaviors::PlayerTag>>,
    attack_query: Query<&crate::behaviors::MeleeAttackState, With<crate::behaviors::PlayerTag>>,
    enemy_query: Query<&Transform, With<crate::behaviors::EnemyTag>>,
    time: Res<Time<Virtual>>,
) {
    use crate::behaviors::*;

    // Don't trigger new melee attacks while already attacking
    if !attack_query.is_empty() {
        return;
    }

    if let Ok((player_entity, player_transform)) = player_query.single() {
        for mut melee in melee_query.iter_mut() {
            // Only tick cooldown if it's not finished (actively cooling down)
            if !melee.cooldown.is_finished() {
                melee.cooldown.tick(time.delta());
            }

            // Find nearest enemy within detection range
            let nearest_enemy = enemy_query.iter()
                .filter(|enemy_transform| {
                    player_transform.translation.distance(enemy_transform.translation) <= melee.detection_range
                })
                .min_by(|a, b| {
                    let dist_a = player_transform.translation.distance(a.translation);
                    let dist_b = player_transform.translation.distance(b.translation);
                    dist_a.partial_cmp(&dist_b).unwrap()
                });

            // Only attack if cooldown is ready AND there's an enemy in range
            if let Some(enemy_transform) = nearest_enemy {
                if melee.cooldown.is_finished() {
                    melee.cooldown.reset();

                    // Calculate initial attack direction
                    let attack_direction = Vec2::new(
                        enemy_transform.translation.x - player_transform.translation.x,
                        enemy_transform.translation.y - player_transform.translation.y,
                    ).normalize();

                    // Add MeleeAttackState to player
                    commands.entity(player_entity).insert(MeleeAttackState {
                        attack_timer: Timer::from_seconds(melee.attack_duration, TimerMode::Once),
                        damage: melee.damage,
                        stun_duration: melee.stun_duration,
                        knockback_force: melee.knockback_force,
                        hitbox_size: melee.hitbox_size,
                        hitbox_color: melee.hitbox_color,
                        attack_direction,
                    });
                }
            }
        }
    }
}

fn execute_melee_attack(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &mut crate::physics::Velocity, &mut crate::behaviors::MeleeAttackState), With<crate::behaviors::PlayerTag>>,
    enemy_query: Query<&Transform, With<crate::behaviors::EnemyTag>>,
    hitbox_query: Query<&crate::behaviors::MeleeHitbox>,
    time: Res<Time<Virtual>>,
) {
    use crate::behaviors::*;

    if let Ok((player_entity, player_transform, mut velocity, mut attack_state)) = player_query.single_mut() {
        // Tick attack timer
        attack_state.attack_timer.tick(time.delta());

        // If hitbox doesn't exist yet, spawn it
        if hitbox_query.is_empty() {
            let angle = attack_state.attack_direction.y.atan2(attack_state.attack_direction.x);
            commands.spawn((
                Sprite {
                    color: Color::srgba(
                        attack_state.hitbox_color.0,
                        attack_state.hitbox_color.1,
                        attack_state.hitbox_color.2,
                        0.3,
                    ),
                    custom_size: Some(Vec2::new(
                        attack_state.hitbox_size.0,
                        attack_state.hitbox_size.1,
                    )),
                    ..default()
                },
                Transform::from_translation(player_transform.translation)
                    .with_rotation(Quat::from_rotation_z(angle)),
                MeleeHitbox {
                    damage: attack_state.damage,
                    stun_duration: attack_state.stun_duration,
                    knockback_force: attack_state.knockback_force,
                    hit_entities: Vec::new(),
                },
            ));
        }

        // Track toward nearest enemy
        const TRACKING_SPEED: f32 = crate::constants::MELEE_TRACKING_SPEED;

        let nearest_enemy = enemy_query.iter()
            .min_by(|a, b| {
                let dist_a = player_transform.translation.distance(a.translation);
                let dist_b = player_transform.translation.distance(b.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some(enemy_transform) = nearest_enemy {
            let direction = Vec2::new(
                enemy_transform.translation.x - player_transform.translation.x,
                enemy_transform.translation.y - player_transform.translation.y,
            );

            let distance = direction.length();

            if distance > 5.0 {
                let normalized_direction = direction.normalize();
                velocity.x = normalized_direction.x * TRACKING_SPEED;
                velocity.y = normalized_direction.y * TRACKING_SPEED;
            } else {
                velocity.x = 0.0;
                velocity.y = 0.0;
            }
        }

        // Check if attack is complete
        if attack_state.attack_timer.just_finished() {
            // Stop movement
            velocity.x = 0.0;
            velocity.y = 0.0;

            // Remove attack state
            commands.entity(player_entity).remove::<MeleeAttackState>();
        }
    }
}

type MeleeHitboxQuery<'w, 's> = Query<'w, 's, (Entity, &'static mut Transform, &'static Sprite, &'static mut crate::behaviors::MeleeHitbox)>;
type MeleePlayerQuery<'w, 's> = Query<'w, 's, &'static Transform, (With<crate::behaviors::PlayerTag>, Without<crate::behaviors::MeleeHitbox>, Without<crate::behaviors::EnemyTag>)>;
type MeleeEnemyQuery<'w, 's> = Query<'w, 's, (Entity, &'static Transform, &'static Sprite, &'static mut crate::physics::Velocity, &'static mut crate::behaviors::Damageable), (With<crate::behaviors::EnemyTag>, Without<crate::behaviors::MeleeHitbox>)>;

fn update_melee_hitboxes(
    mut commands: Commands,
    mut hitbox_query: MeleeHitboxQuery,
    player_query: MeleePlayerQuery,
    attack_state_query: Query<&crate::behaviors::MeleeAttackState, With<crate::behaviors::PlayerTag>>,
    mut enemy_query: MeleeEnemyQuery,
) {
    use crate::behaviors::*;

    // Remove hitboxes if attack state is gone
    if attack_state_query.is_empty() {
        for (hitbox_entity, _, _, _) in hitbox_query.iter() {
            commands.entity(hitbox_entity).despawn();
        }
        return;
    }

    // Update hitbox position and check for hits
    if let Ok(player_transform) = player_query.single() {
        for (_hitbox_entity, mut hitbox_transform, hitbox_sprite, mut hitbox) in hitbox_query.iter_mut() {
            // Keep hitbox centered on player
            hitbox_transform.translation = player_transform.translation;

            let hitbox_size = hitbox_sprite.custom_size.unwrap_or(Vec2::ONE);

            // Check collision with all enemies
            for (enemy_entity, enemy_transform, enemy_sprite, mut enemy_velocity, mut damageable) in enemy_query.iter_mut() {
                // Skip if already hit this entity
                if hitbox.hit_entities.contains(&enemy_entity) {
                    continue;
                }

                let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);

                // Check AABB collision
                if check_collision(
                    hitbox_transform.translation,
                    hitbox_size,
                    enemy_transform.translation,
                    enemy_size,
                ) {
                    // Apply damage
                    damageable.health -= hitbox.damage;

                    // Apply knockback
                    let knockback_direction = Vec2::new(
                        enemy_transform.translation.x - player_transform.translation.x,
                        enemy_transform.translation.y - player_transform.translation.y,
                    ).normalize_or_zero();

                    enemy_velocity.x = knockback_direction.x * hitbox.knockback_force;
                    enemy_velocity.y = knockback_direction.y * hitbox.knockback_force;

                    // Apply stun
                    commands.entity(enemy_entity).insert(Stunned {
                        timer: Timer::from_seconds(hitbox.stun_duration, TimerMode::Once),
                    });

                    // Mark as hit
                    hitbox.hit_entities.push(enemy_entity);
                }
            }
        }
    }
}

fn update_stunned_enemies(
    mut commands: Commands,
    mut stunned_query: Query<(Entity, &mut crate::behaviors::Stunned)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut stunned) in stunned_query.iter_mut() {
        stunned.timer.tick(time.delta());

        if stunned.timer.just_finished() {
            commands.entity(entity).remove::<crate::behaviors::Stunned>();
        }
    }
}

fn check_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    pos1.x - half_size1.x < pos2.x + half_size2.x
        && pos1.x + half_size1.x > pos2.x - half_size2.x
        && pos1.y - half_size1.y < pos2.y + half_size2.y
        && pos1.y + half_size1.y > pos2.y - half_size2.y
}

// ============ Weapon Cooldown UI Systems ============

struct BarLayout {
	width: f32,
	height: f32,
	start_y: f32,
	spacing: f32,
}

type NewProjectileWeaponsQuery<'w, 's> = Query<'w, 's, (Entity, &'static WeaponName), (With<crate::behaviors::ProjectileSpawner>, Without<HasCooldownUI>)>;
type NewMeleeWeaponsQuery<'w, 's> = Query<'w, 's, (Entity, &'static WeaponName), (With<crate::behaviors::MeleeAttack>, Without<HasCooldownUI>)>;

fn spawn_weapon_cooldown_bars(
    mut commands: Commands,
    projectile_weapons: NewProjectileWeaponsQuery,
    melee_weapons: NewMeleeWeaponsQuery,
    existing_projectile_weapons: Query<Entity, (With<crate::behaviors::ProjectileSpawner>, With<HasCooldownUI>)>,
    existing_melee_weapons: Query<Entity, (With<crate::behaviors::MeleeAttack>, With<HasCooldownUI>)>,
) {
    const LAYOUT: BarLayout = BarLayout {
		width: 200.0,
		height: 15.0,
		start_y: 10.0,
		spacing: 25.0,
	};

    // Start bar index after existing weapons
    let mut bar_index = existing_projectile_weapons.iter().count() + existing_melee_weapons.iter().count();

    // Spawn bars for projectile weapons
    for (entity, weapon_name) in projectile_weapons.iter() {
        spawn_cooldown_bar(&mut commands, entity, &weapon_name.0, bar_index, &LAYOUT);
        bar_index += 1;
    }

    // Spawn bars for melee weapons
    for (entity, weapon_name) in melee_weapons.iter() {
        spawn_cooldown_bar(&mut commands, entity, &weapon_name.0, bar_index, &LAYOUT);
        bar_index += 1;
    }
}

fn spawn_cooldown_bar(
    commands: &mut Commands,
    weapon_entity: Entity,
    weapon_name: &str,
    index: usize,
    layout: &BarLayout,
) {
    let y_position = layout.start_y + (index as f32 * layout.spacing);

    // Mark weapon as having UI
    commands.entity(weapon_entity).insert(HasCooldownUI);

    // Spawn background bar
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(y_position),
            right: Val::Px(10.0),
            width: Val::Px(layout.width),
            height: Val::Px(layout.height),
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        ZIndex(10),
        WeaponCooldownBar { weapon_entity },
        WeaponCooldownBarBackground,
    ));

    // Spawn foreground bar (fills up as cooldown progresses)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(y_position),
            right: Val::Px(10.0),
            width: Val::Px(0.0),
            height: Val::Px(layout.height),
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
        ZIndex(11),
        WeaponCooldownBar { weapon_entity },
        WeaponCooldownBarForeground,
    ));

    // Spawn text label
    commands.spawn((
        Text::new(weapon_name),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(y_position - 2.0),
            right: Val::Px(15.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        ZIndex(12),
        WeaponCooldownBar { weapon_entity },
        WeaponCooldownText,
    ));
}

fn update_weapon_cooldown_bars(
    projectile_weapons: Query<(Entity, &crate::behaviors::ProjectileSpawner)>,
    melee_weapons: Query<(Entity, &crate::behaviors::MeleeAttack)>,
    mut bars: Query<(&WeaponCooldownBar, &mut Node), With<WeaponCooldownBarForeground>>,
) {
    const BAR_WIDTH: f32 = 200.0;

    for (bar, mut node) in bars.iter_mut() {
        // Check if it's a projectile weapon
        if let Ok((_, spawner)) = projectile_weapons.get(bar.weapon_entity) {
            // Full bar when ready, empty when just fired, fills as it cools down
            let readiness = if spawner.cooldown.is_finished() {
                1.0
            } else {
                spawner.cooldown.fraction()
            };
            node.width = Val::Px(BAR_WIDTH * readiness);
            continue;
        }

        // Check if it's a melee weapon
        if let Ok((_, melee)) = melee_weapons.get(bar.weapon_entity) {
            // Full bar when ready, empty when just fired, fills as it cools down
            let readiness = if melee.cooldown.is_finished() {
                1.0
            } else {
                melee.cooldown.fraction()
            };
            node.width = Val::Px(BAR_WIDTH * readiness);
        }
    }
}
