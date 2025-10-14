use bevy::prelude::*;

pub fn detect_melee_targets(
	mut commands: Commands,
	mut melee_query: Query<
		&mut crate::behaviors::MeleeAttack,
		With<crate::behaviors::FollowPlayer>,
	>,
	player_query: Query<(Entity, &Transform), With<crate::behaviors::PlayerTag>>,
	attack_query: Query<&crate::behaviors::MeleeAttackState, With<crate::behaviors::PlayerTag>>,
	enemy_query: Query<&Transform, With<crate::behaviors::EnemyTag>>,
	mut player_energy_query: Query<&mut crate::behaviors::PlayerEnergy, With<crate::behaviors::PlayerTag>>,
	active_weapon: Res<crate::weapons::ActiveWeaponState>,
	time: Res<Time<Virtual>>,
) {
	use crate::behaviors::*;

	// Don't trigger new melee attacks while already attacking
	if !attack_query.is_empty() {
		return;
	}

	if let Ok((player_entity, player_transform)) = player_query.single() {
		for mut melee in melee_query.iter_mut() {
			// Always tick cooldown if it's not finished (actively cooling down)
			if !melee.cooldown.is_finished() {
				melee.cooldown.tick(time.delta());
			}

			// Only allow attacking if melee weapon is active
			if active_weapon.active_slot != Some(WeaponSlot::Melee) {
				continue;
			}

			// Find nearest enemy within detection range
			let nearest_enemy = enemy_query
				.iter()
				.filter(|enemy_transform| {
					player_transform
						.translation
						.distance(enemy_transform.translation)
						<= melee.detection_range
				})
				.min_by(|a, b| {
					let dist_a = player_transform.translation.distance(a.translation);
					let dist_b = player_transform.translation.distance(b.translation);
					dist_a.partial_cmp(&dist_b).unwrap()
				});

			// Only attack if cooldown is ready AND there's an enemy in range
			if let Some(enemy_transform) = nearest_enemy {
				if melee.cooldown.is_finished() {
					// Check if player has enough energy
					if let Ok(mut player_energy) = player_energy_query.single_mut() {
						if player_energy.current < melee.energy_cost {
							continue; // Not enough energy, skip attack
						}
						player_energy.current -= melee.energy_cost;
					}

					melee.cooldown.reset();

					// Calculate initial attack direction
					let attack_direction = Vec2::new(
						enemy_transform.translation.x - player_transform.translation.x,
						enemy_transform.translation.y - player_transform.translation.y,
					)
					.normalize();

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

pub fn execute_melee_attack(
	mut commands: Commands,
	mut player_query: Query<
		(
			Entity,
			&Transform,
			&mut crate::physics::Velocity,
			&mut crate::behaviors::MeleeAttackState,
		),
		With<crate::behaviors::PlayerTag>,
	>,
	enemy_query: Query<&Transform, With<crate::behaviors::EnemyTag>>,
	hitbox_query: Query<&crate::behaviors::MeleeHitbox>,
	time: Res<Time<Virtual>>,
) {
	use crate::behaviors::*;

	if let Ok((player_entity, player_transform, mut velocity, mut attack_state)) =
		player_query.single_mut()
	{
		// Tick attack timer
		attack_state.attack_timer.tick(time.delta());

		// If hitbox doesn't exist yet, spawn it
		if hitbox_query.is_empty() {
			let angle = attack_state
				.attack_direction
				.y
				.atan2(attack_state.attack_direction.x);
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

		let nearest_enemy = enemy_query.iter().min_by(|a, b| {
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

type MeleeHitboxQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static mut Transform,
		&'static Sprite,
		&'static mut crate::behaviors::MeleeHitbox,
	),
>;
type MeleePlayerQuery<'w, 's> = Query<
	'w,
	's,
	&'static Transform,
	(
		With<crate::behaviors::PlayerTag>,
		Without<crate::behaviors::MeleeHitbox>,
		Without<crate::behaviors::EnemyTag>,
	),
>;
type MeleeEnemyQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static Transform,
		&'static Sprite,
		&'static mut crate::physics::Velocity,
		&'static mut crate::behaviors::Damageable,
	),
	(
		With<crate::behaviors::EnemyTag>,
		Without<crate::behaviors::MeleeHitbox>,
	),
>;

pub fn update_melee_hitboxes(
	mut commands: Commands,
	mut hitbox_query: MeleeHitboxQuery,
	player_query: MeleePlayerQuery,
	attack_state_query: Query<
		&crate::behaviors::MeleeAttackState,
		With<crate::behaviors::PlayerTag>,
	>,
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
		for (_hitbox_entity, mut hitbox_transform, hitbox_sprite, mut hitbox) in
			hitbox_query.iter_mut()
		{
			// Keep hitbox centered on player
			hitbox_transform.translation = player_transform.translation;

			let hitbox_size = hitbox_sprite.custom_size.unwrap_or(Vec2::ONE);

			// Check collision with all enemies
			for (enemy_entity, enemy_transform, enemy_sprite, mut enemy_velocity, mut damageable) in
				enemy_query.iter_mut()
			{
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
					)
					.normalize_or_zero();

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

pub fn update_stunned_enemies(
	mut commands: Commands,
	mut stunned_query: Query<(Entity, &mut crate::behaviors::Stunned)>,
	time: Res<Time<Virtual>>,
) {
	for (entity, mut stunned) in stunned_query.iter_mut() {
		stunned.timer.tick(time.delta());

		if stunned.timer.just_finished() {
			commands
				.entity(entity)
				.remove::<crate::behaviors::Stunned>();
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
