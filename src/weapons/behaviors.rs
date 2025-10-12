use bevy::prelude::*;
use std::f32::consts::PI;

#[derive(Resource, Default)]
pub struct OrbitingEntityCount(pub usize);

// Generic update system for orbiting entities
pub fn update_orbiting_entities(
	mut orbiting_query: Query<(
		&mut Transform,
		&mut crate::behaviors::OrbitingBehavior,
		&crate::behaviors::FollowPlayer,
	)>,
	player_query: Query<
		&Transform,
		(
			With<crate::behaviors::PlayerTag>,
			Without<crate::behaviors::OrbitingBehavior>,
		),
	>,
	time: Res<Time<Virtual>>,
) {
	if let Ok(player_transform) = player_query.single() {
		for (mut transform, mut behavior, _) in orbiting_query.iter_mut() {
			// Rotate based on speed
			behavior.angle += behavior.speed * time.delta_secs();

			// Update position relative to player
			transform.translation.x =
				player_transform.translation.x + behavior.angle.cos() * behavior.radius;
			transform.translation.y =
				player_transform.translation.y + behavior.angle.sin() * behavior.radius;
			transform.rotation = Quat::from_rotation_z(behavior.angle + PI / 2.0);
		}
	}
}

// System to redistribute orbiting entities when new ones are added
pub fn redistribute_orbiting_entities(
	mut all_orbiting: Query<
		&mut crate::behaviors::OrbitingBehavior,
		With<crate::behaviors::FollowPlayer>,
	>,
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
pub fn update_projectile_spawners(
	mut commands: Commands,
	mut spawner_query: Query<(
		&Transform,
		&mut crate::behaviors::ProjectileSpawner,
		Has<crate::behaviors::PlayerTag>,
		Has<crate::behaviors::EnemyTag>,
	)>,
	player_query: Query<
		&Transform,
		(
			With<crate::behaviors::PlayerTag>,
			Without<crate::behaviors::ProjectileSpawner>,
		),
	>,
	enemy_query: Query<
		&Transform,
		(
			With<crate::behaviors::EnemyTag>,
			Without<crate::behaviors::ProjectileSpawner>,
		),
	>,
	time: Res<Time<Virtual>>,
) {
	use crate::behaviors::*;

	for (spawner_transform, mut spawner, is_player_weapon, is_enemy) in spawner_query.iter_mut() {
		// Only tick if not finished (actively cooling down)
		if !spawner.cooldown.is_finished() {
			spawner.cooldown.tick(time.delta());
			continue; // Skip to next weapon while cooling down
		}

		// Cooldown is ready, try to fire
		let spawn_direction = match &spawner.spawn_logic {
			SpawnLogic::NearestEnemy => {
				// For player weapons, target enemies. For enemy weapons, target player.
				if is_player_weapon {
					// Find nearest enemy (optionally within range)
					let nearest_enemy = enemy_query
						.iter()
						.filter(|enemy_transform| {
							// If fire_range is set, only consider enemies within range
							if let Some(range) = spawner.fire_range {
								spawner_transform
									.translation
									.distance(enemy_transform.translation)
									<= range
							} else {
								true // No range limit
							}
						})
						.min_by(|a, b| {
							let dist_a = spawner_transform.translation.distance(a.translation);
							let dist_b = spawner_transform.translation.distance(b.translation);
							dist_a.partial_cmp(&dist_b).unwrap()
						});

					// If no enemy in range, don't fire
					if let Some(enemy_transform) = nearest_enemy {
						let direction = Vec2::new(
							enemy_transform.translation.x - spawner_transform.translation.x,
							enemy_transform.translation.y - spawner_transform.translation.y,
						);
						Some(direction.normalize())
					} else {
						// No enemy in range, skip spawning projectile
						None
					}
				} else if is_enemy {
					// Enemy targeting player
					if let Ok(player_transform) = player_query.single() {
						let direction = Vec2::new(
							player_transform.translation.x - spawner_transform.translation.x,
							player_transform.translation.y - spawner_transform.translation.y,
						);
						let distance = direction.length();

						// Check fire range
						if let Some(range) = spawner.fire_range {
							if distance > range {
								None
							} else {
								Some(direction.normalize())
							}
						} else {
							Some(direction.normalize())
						}
					} else {
						None
					}
				} else {
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

		// Determine target filter based on who's spawning
		let target_filter = if is_player_weapon {
			TargetFilter::Enemies
		} else {
			TargetFilter::Player
		};

		commands.spawn((
			Sprite {
				color: Color::srgb(template.color.0, template.color.1, template.color.2),
				custom_size: Some(Vec2::new(template.size.0, template.size.1)),
				..default()
			},
			Transform::from_xyz(
				spawner_transform.translation.x + direction.x * 30.0,
				spawner_transform.translation.y + direction.y * 30.0,
				0.0,
			)
			.with_rotation(Quat::from_rotation_z(angle)),
			crate::physics::Velocity {
				x: direction.x * template.speed,
				y: direction.y * template.speed,
			},
			DamageOnContact {
				damage: template.damage,
				damage_type: DamageType::OneTime,
				targets: target_filter,
			},
			DespawnOnTimer {
				timer: Timer::from_seconds(template.lifetime, TimerMode::Once),
			},
			ProjectileTag,
		));
	}
}

// Generic despawn timer system
pub fn update_despawn_timers(
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
