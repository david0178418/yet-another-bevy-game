use bevy::prelude::*;

pub struct PlayerPlugin;

type PlayerStatsQuery<'w, 's> = Query<
	'w,
	's,
	(&'static Player, &'static crate::behaviors::Damageable),
	Or<(Changed<Player>, Changed<crate::behaviors::Damageable>)>,
>;

type ChargingPlayerQuery<'w, 's> = Query<
	'w,
	's,
	(
		&'static mut crate::behaviors::PlayerEnergy,
		&'static mut crate::physics::Velocity,
	),
	(With<Player>, With<crate::behaviors::EnergyCharging>),
>;

type RepulsionPlayerQuery<'w, 's> = Query<
	'w,
	's,
	(&'static Transform, &'static crate::behaviors::PlayerEnergy),
	(With<Player>, With<crate::behaviors::EnergyCharging>),
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
	With<Player>,
>;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				// Process input before physics for minimal latency
				player_movement,
				player_jump,
				handle_energy_charging_input,
			)
				.before(crate::physics::PhysicsSet),
		)
		.add_systems(
			Update,
			(
				spawn_player,
				spawn_initial_weapon,
				update_player_stats_display,
				update_xp_bar,
				regenerate_energy,
				charge_energy,
				update_energy_bar,
			),
		)
		.add_systems(
			Update,
			apply_repulsion_field.after(crate::movement::MovementSystemSet),
		);
	}
}

#[derive(Component)]
struct NeedsInitialWeapons {
	weapons: Vec<crate::InitialWeapon>,
}

#[derive(Component)]
pub struct Player {
	pub speed: f32,
	pub jump_force: f32,
	pub level: u32,
}

impl Default for Player {
	fn default() -> Self {
		Self {
			speed: crate::constants::PLAYER_DEFAULT_SPEED,
			jump_force: crate::constants::PLAYER_DEFAULT_JUMP_FORCE,
			level: 1,
		}
	}
}

#[derive(Component)]
struct PlayerStatsText;

#[derive(Component)]
struct XPBarBackground;

#[derive(Component)]
struct XPBarForeground;

#[derive(Component)]
struct XPText;

#[derive(Component)]
struct EnergyBarBackground;

#[derive(Component)]
struct EnergyBarForeground;

#[derive(Component)]
struct EnergyText;

#[derive(Component)]
struct RepulsionFieldIndicator;

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

fn spawn_platforms(commands: &mut Commands) {
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

fn spawn_player_ui(commands: &mut Commands) {
	use crate::constants::*;

	commands.spawn((
		Text::new("Health: 100/100 | Level: 1"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(UI_MARGIN),
			left: Val::Px(UI_MARGIN),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_NORMAL,
			..default()
		},
		PlayerStatsText,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(XP_BAR_WIDTH),
			height: Val::Px(XP_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(XP_BAR_COLOR_BG),
		ZIndex(0),
		XPBarBackground,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(0.0),
			height: Val::Px(XP_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(XP_BAR_COLOR_FG),
		ZIndex(1),
		XPBarForeground,
	));

	commands.spawn((
		Text::new("XP: 0/100"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP + 2.0),
			left: Val::Px(UI_MARGIN + 5.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_SMALL,
			..default()
		},
		ZIndex(2),
		XPText,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(ENERGY_BAR_WIDTH),
			height: Val::Px(ENERGY_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(ENERGY_BAR_COLOR_BG),
		ZIndex(0),
		EnergyBarBackground,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(ENERGY_BAR_WIDTH),
			height: Val::Px(ENERGY_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(ENERGY_BAR_COLOR_FG),
		ZIndex(1),
		EnergyBarForeground,
	));

	commands.spawn((
		Text::new("Energy: 100/100"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP + 2.0),
			left: Val::Px(UI_MARGIN + 5.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_SMALL,
			..default()
		},
		ZIndex(2),
		EnergyText,
	));
}

fn spawn_player(
	mut commands: Commands,
	game_config: Option<Res<crate::GameConfig>>,
	config_assets: Res<Assets<crate::GameConfigData>>,
	player_query: Query<(), With<Player>>,
	ui_query: Query<(), With<PlayerStatsText>>,
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
		Player::default(),
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
		spawn_player_ui(&mut commands);
	}
}

fn spawn_initial_weapon(
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

fn player_movement(
	keyboard: Res<ButtonInput<KeyCode>>,
	gamepads: Query<&Gamepad>,
	mut query: Query<(&mut crate::physics::Velocity, &Player), Without<crate::behaviors::EnergyCharging>>,
	time: Res<Time>, // Use real time for input, not virtual (paused) time
) {
	for (mut velocity, player) in query.iter_mut() {
		let mut direction = 0.0;

		// Keyboard input
		if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
			direction -= 1.0;
		}
		if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
			direction += 1.0;
		}

		// Gamepad input
		for gamepad in gamepads.iter() {
			// Left stick X axis
			if let Some(axis_value) = gamepad.get(GamepadAxis::LeftStickX) {
				if axis_value.abs() > crate::constants::GAMEPAD_DEADZONE {
					direction = axis_value;
				}
			}

			// D-pad as alternative
			if gamepad.pressed(GamepadButton::DPadLeft) {
				direction = -1.0;
			}
			if gamepad.pressed(GamepadButton::DPadRight) {
				direction = 1.0;
			}
		}

		// Acceleration-based movement
		let target_speed = direction * player.speed;
		let speed_diff = target_speed - velocity.x;

		if speed_diff.abs() > 0.01 {
			// Choose acceleration or deceleration based on input
			let accel = if direction.abs() > 0.01 {
				crate::constants::PLAYER_ACCELERATION
			} else {
				crate::constants::PLAYER_DECELERATION
			};

			let change = speed_diff.signum() * accel * time.delta_secs();

			// Snap to target if close enough, otherwise apply acceleration
			if speed_diff.abs() <= change.abs() {
				velocity.x = target_speed;
			} else {
				velocity.x += change;
			}
		}
	}
}

fn player_jump(
	keyboard: Res<ButtonInput<KeyCode>>,
	gamepads: Query<&Gamepad>,
	mut query: Query<
		(
			&mut crate::physics::Velocity,
			&Player,
			&crate::physics::Grounded,
		),
		Without<crate::behaviors::EnergyCharging>,
	>,
	powerup_state: Res<crate::powerups::PowerupState>,
) {
	// Don't process jump input while menu is showing
	if powerup_state.showing {
		return;
	}

	for (mut velocity, player, grounded) in query.iter_mut() {
		let mut should_jump = false;

		// Keyboard input
		if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::KeyW) {
			should_jump = true;
		}

		// Gamepad input (South button - typically A on Xbox, Cross on PlayStation)
		for gamepad in gamepads.iter() {
			if gamepad.just_pressed(GamepadButton::South) {
				should_jump = true;
			}
		}

		if should_jump && grounded.0 {
			velocity.y = player.jump_force;
		}
	}
}

fn update_player_stats_display(
	player_query: PlayerStatsQuery,
	mut text_query: Query<&mut Text, With<PlayerStatsText>>,
) {
	if let Ok((player, damageable)) = player_query.single() {
		if let Ok(mut text) = text_query.single_mut() {
			**text = format!(
				"Health: {:.0}/{:.0} | Level: {}",
				damageable.health, damageable.max_health, player.level
			);
		}
	}
}

fn update_xp_bar(
	player_xp: Res<crate::experience::PlayerExperience>,
	mut xp_bar_query: Query<&mut Node, With<XPBarForeground>>,
	mut xp_text_query: Query<&mut Text, With<XPText>>,
) {
	// Only update if UI exists
	let Ok(mut node) = xp_bar_query.single_mut() else {
		return;
	};

	let Ok(mut text) = xp_text_query.single_mut() else {
		return;
	};

	// Update XP bar width
	let xp_percent =
		(player_xp.current_xp as f32 / player_xp.xp_to_next_level as f32).clamp(0.0, 1.0);
	let new_width = crate::constants::XP_BAR_WIDTH * xp_percent;
	node.width = Val::Px(new_width);

	// Update XP text
	**text = format!(
		"XP: {}/{}",
		player_xp.current_xp, player_xp.xp_to_next_level
	);
}

fn regenerate_energy(
	mut player_query: Query<&mut crate::behaviors::PlayerEnergy, With<Player>>,
	time: Res<Time<Virtual>>,
) {
	for mut energy in player_query.iter_mut() {
		energy.current = (energy.current + energy.regen_rate * time.delta_secs()).min(energy.max);
	}
}

fn handle_energy_charging_input(
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
				const REPULSION_RANGE: f32 = crate::constants::REPULSION_RANGE;
				const NUM_RINGS: usize = 8;

				// Spawn multiple concentric circles with gradient transparency
				for i in 0..NUM_RINGS {
					let ring_index = i as f32;
					let ring_fraction = (ring_index + 1.0) / NUM_RINGS as f32;

					// Outer rings are more transparent (reduced overall alpha)
					let alpha = 0.05 * (1.0 - ring_fraction * 0.8);

					// Each ring is progressively larger
					let radius = REPULSION_RANGE * ring_fraction;

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

fn charge_energy(mut player_query: ChargingPlayerQuery, time: Res<Time<Virtual>>) {
	for (mut energy, mut velocity) in player_query.iter_mut() {
		// Accumulate energy at fast rate
		energy.current = (energy.current + crate::constants::ENERGY_CHARGE_RATE * time.delta_secs()).min(energy.max);

		// Ensure velocity stays at 0 (belt and suspenders approach)
		velocity.x = 0.0;
		velocity.y = 0.0;
	}
}

fn apply_repulsion_field(
	player_query: RepulsionPlayerQuery,
	mut enemy_query: Query<(&Transform, &mut crate::physics::Velocity, &crate::behaviors::Damageable), With<crate::behaviors::EnemyTag>>,
	time: Res<Time<Virtual>>,
) {
	const MAX_RANGE: f32 = crate::constants::REPULSION_RANGE;
	const BASE_FORCE: f32 = crate::constants::REPULSION_BASE_FORCE;

	// Only apply if player is charging
	if let Ok((player_transform, player_energy)) = player_query.single() {
		// Skip if repulsion force is zero (no powerup acquired yet)
		if player_energy.repulsion_force <= 0.0 {
			return;
		}

		// Apply repulsion force to all enemies within range
		// Force formula: (powerup_level * base * distance_falloff) / enemy_max_health
		// This ensures: closer enemies feel more force, tankier enemies resist better
		for (enemy_transform, mut enemy_velocity, enemy_damageable) in enemy_query.iter_mut() {
			// Calculate distance to player
			let direction_to_enemy = Vec2::new(
				enemy_transform.translation.x - player_transform.translation.x,
				enemy_transform.translation.y - player_transform.translation.y,
			);
			let distance = direction_to_enemy.length();

			// Only apply force within range
			if !(0.1..=MAX_RANGE).contains(&distance) {
				continue;
			}

			// Normalize direction (away from player)
			let direction = direction_to_enemy / distance;

			// Distance-based falloff (closer = stronger)
			let distance_factor = 1.0 - (distance / MAX_RANGE);

			// Calculate force magnitude scaled by enemy max health
			// Stronger enemies (more health) feel less force
			let force_magnitude = (player_energy.repulsion_force * BASE_FORCE * distance_factor)
				/ enemy_damageable.max_health;

			// Apply force to enemy velocity
			enemy_velocity.x += direction.x * force_magnitude * time.delta_secs();
			enemy_velocity.y += direction.y * force_magnitude * time.delta_secs();
		}
	}
}

fn update_energy_bar(
	player_query: Query<&crate::behaviors::PlayerEnergy, (With<Player>, Changed<crate::behaviors::PlayerEnergy>)>,
	mut energy_bar_query: Query<&mut Node, With<EnergyBarForeground>>,
	mut energy_text_query: Query<&mut Text, With<EnergyText>>,
) {
	// Only update if UI exists
	let Ok(mut node) = energy_bar_query.single_mut() else {
		return;
	};

	let Ok(mut text) = energy_text_query.single_mut() else {
		return;
	};

	// Only update if energy changed
	if let Ok(energy) = player_query.single() {
		// Update energy bar width
		let energy_percent = (energy.current / energy.max).clamp(0.0, 1.0);
		let new_width = crate::constants::ENERGY_BAR_WIDTH * energy_percent;
		node.width = Val::Px(new_width);

		// Update energy text
		**text = format!("Energy: {:.0}/{:.0}", energy.current, energy.max);
	}
}
