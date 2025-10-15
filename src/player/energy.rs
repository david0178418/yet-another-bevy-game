use bevy::prelude::*;

type ChargingPlayerQuery<'w, 's> = Query<
	'w,
	's,
	(
		&'static mut crate::behaviors::PlayerEnergy,
		&'static mut crate::physics::Velocity,
	),
	(With<super::Player>, With<crate::behaviors::EnergyCharging>),
>;

type RepulsionPlayerQuery<'w, 's> = Query<
	'w,
	's,
	(&'static Transform, &'static crate::behaviors::PlayerEnergy),
	(With<super::Player>, With<crate::behaviors::EnergyCharging>),
>;

type ChargingInputPlayerQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static Transform,
		&'static mut crate::physics::Velocity,
		Has<crate::behaviors::EnergyCharging>,
		&'static crate::behaviors::PlayerEnergy,
	),
	With<super::Player>,
>;

type RepulsionEnemyQuery<'w, 's> = Query<
	'w,
	's,
	(
		Entity,
		&'static Transform,
		&'static mut crate::physics::Velocity,
		&'static crate::behaviors::Damageable,
		Has<crate::behaviors::FlyingMovement>,
	),
	With<crate::behaviors::EnemyTag>,
>;

#[derive(Component)]
pub struct RepulsionFieldIndicator;

pub fn regenerate_energy(
	mut player_query: Query<&mut crate::behaviors::PlayerEnergy, With<super::Player>>,
	time: Res<Time<Virtual>>,
) {
	for mut energy in player_query.iter_mut() {
		energy.current = (energy.current + energy.regen_rate * time.delta_secs()).min(energy.max);
	}
}

pub fn handle_energy_charging_input(
	mut commands: Commands,
	keyboard: Res<ButtonInput<KeyCode>>,
	gamepads: Query<&Gamepad>,
	mut player_query: ChargingInputPlayerQuery,
	indicator_query: Query<Entity, With<RepulsionFieldIndicator>>,
	powerup_state: Res<crate::powerups::PowerupState>,
	(mut meshes, mut materials): (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
) {
	// Don't process input while menu is showing
	if powerup_state.showing {
		return;
	}

	for (player_entity, player_transform, mut velocity, is_charging, player_energy) in player_query.iter_mut() {
		let mut charging_input = false;

		// Check keyboard (F key)
		if keyboard.pressed(KeyCode::KeyF) {
			charging_input = true;
		}

		// Check gamepad (Y button / North)
		for gamepad in gamepads.iter() {
			if gamepad.pressed(GamepadButton::North) {
				charging_input = true;
			}
		}

		if charging_input && !is_charging {
			// Start charging
			commands.entity(player_entity).insert(crate::behaviors::EnergyCharging);
			velocity.x = 0.0;
			velocity.y = 0.0;

			// Spawn repulsion field indicators (only if player has repulsion force)
			if player_energy.repulsion_force > 0.0 {
				const MAX_RANGE: f32 = crate::constants::REPULSION_RANGE;
				const MIN_RANGE: f32 = crate::constants::MIN_REPULSION_RANGE;
				const MAX_FORCE: f32 = crate::constants::MAX_REPULSION_FORCE;
				const NUM_RINGS: usize = 8;

				// Calculate effective range based on current repulsion force
				let force_ratio = (player_energy.repulsion_force / MAX_FORCE).min(1.0);
				let effective_range = MIN_RANGE + (MAX_RANGE - MIN_RANGE) * force_ratio;

				// Spawn multiple concentric circles with gradient transparency
				for i in 0..NUM_RINGS {
					let ring_index = i as f32;
					let ring_fraction = (ring_index + 1.0) / NUM_RINGS as f32;

					// Outer rings are more transparent (reduced overall alpha)
					let alpha = 0.05 * (1.0 - ring_fraction * 0.8);

					// Each ring is progressively larger
					let radius = effective_range * ring_fraction;

					// Create a circle mesh
					let circle_mesh = Circle::new(radius);
					let mesh_handle = meshes.add(circle_mesh);

					// Create a semi-transparent blue material
					let color = Color::srgba(0.3, 0.6, 1.0, alpha);
					let material_handle = materials.add(ColorMaterial::from(color));

					commands.spawn((
						Mesh2d(mesh_handle),
						MeshMaterial2d(material_handle),
						Transform::from_translation(player_transform.translation.with_z(-1.0)),
						RepulsionFieldIndicator,
					));
				}
			}
		} else if !charging_input && is_charging {
			// Stop charging
			commands.entity(player_entity).remove::<crate::behaviors::EnergyCharging>();

			// Despawn all repulsion field indicators
			for indicator_entity in indicator_query.iter() {
				commands.entity(indicator_entity).despawn();
			}
		}
	}
}

pub fn charge_energy(mut player_query: ChargingPlayerQuery, time: Res<Time<Virtual>>) {
	for (mut energy, mut velocity) in player_query.iter_mut() {
		// Accumulate energy at fast rate
		energy.current = (energy.current + crate::constants::ENERGY_CHARGE_RATE * time.delta_secs()).min(energy.max);

		// Ensure velocity stays at 0 (belt and suspenders approach)
		velocity.x = 0.0;
		velocity.y = 0.0;
	}
}

pub fn apply_repulsion_field(
	mut commands: Commands,
	player_query: RepulsionPlayerQuery,
	mut enemy_query: RepulsionEnemyQuery,
) {
	const MAX_RANGE: f32 = crate::constants::REPULSION_RANGE;
	const MIN_RANGE: f32 = crate::constants::MIN_REPULSION_RANGE;
	const MAX_FORCE: f32 = crate::constants::MAX_REPULSION_FORCE;
	const BASE_SPEED: f32 = crate::constants::REPULSION_BASE_SPEED;

	// Only apply if player is charging
	if let Ok((player_transform, player_energy)) = player_query.single() {
		// Skip if repulsion force is zero (no powerup acquired yet)
		if player_energy.repulsion_force <= 0.0 {
			return;
		}

		// Calculate effective range based on current repulsion force
		let force_ratio = (player_energy.repulsion_force / MAX_FORCE).min(1.0);
		let effective_range = MIN_RANGE + (MAX_RANGE - MIN_RANGE) * force_ratio;

		// Apply repulsion velocity to all enemies within range
		// Speed formula: (powerup_level * base_speed * distance_falloff) / sqrt(enemy_max_health)
		// This ensures: closer enemies pushed harder, tankier enemies resist better
		for (enemy_entity, enemy_transform, mut enemy_velocity, enemy_damageable, is_flying) in
			enemy_query.iter_mut()
		{
			// Calculate distance to player
			let direction_to_enemy = Vec2::new(
				enemy_transform.translation.x - player_transform.translation.x,
				enemy_transform.translation.y - player_transform.translation.y,
			);
			let distance = direction_to_enemy.length();

			// Only apply repulsion within effective range
			if !(0.1..=effective_range).contains(&distance) {
				continue;
			}

			// Mark enemy as in repulsion field (stops movement behaviors)
			commands.entity(enemy_entity).insert(crate::behaviors::InRepulsionField);

			// Normalize direction (away from player)
			let direction = direction_to_enemy / distance;

			// Distance-based falloff (closer = stronger push)
			let distance_factor = 1.0 - (distance / effective_range);

			// Calculate repulsion speed scaled by enemy max health (sqrt for gentler scaling)
			// Tankier enemies resist better but still get pushed
			let repulsion_speed =
				(player_energy.repulsion_force * BASE_SPEED * distance_factor) / enemy_damageable.max_health.sqrt();

			// Set velocity directly (replaces movement behavior velocity)
			enemy_velocity.x = direction.x * repulsion_speed;

			// Only set Y velocity for flying entities; grounded entities use gravity
			if is_flying {
				enemy_velocity.y = direction.y * repulsion_speed;
			}
		}
	}
}

pub fn cleanup_repulsion_markers(
	mut commands: Commands,
	player_query: Query<(), (With<super::Player>, Without<crate::behaviors::EnergyCharging>)>,
	marked_enemies: Query<Entity, With<crate::behaviors::InRepulsionField>>,
) {
	// Only cleanup if player is NOT charging
	if player_query.is_empty() {
		return; // Player doesn't exist or is charging
	}

	// Remove InRepulsionField marker from all marked enemies
	for enemy_entity in marked_enemies.iter() {
		commands
			.entity(enemy_entity)
			.remove::<crate::behaviors::InRepulsionField>();
	}
}
