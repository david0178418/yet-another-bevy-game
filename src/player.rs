use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
                spawn_player,
                spawn_initial_weapon,
                player_movement,
                player_jump,
                update_player_stats_display,
                update_xp_bar,
            ));
    }
}

#[derive(Component)]
struct NeedsInitialWeapon {
	weapon_id: String,
	count: u32,
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
            speed: 200.0,
            jump_force: 400.0,
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

const PLATFORM_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);

fn spawn_platform(commands: &mut Commands, position: Vec3, size: Vec2) {
	commands.spawn((
		Sprite {
			color: PLATFORM_COLOR,
			custom_size: Some(size),
			..default()
		},
		Transform::from_translation(position),
		crate::physics::Ground,
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

	PLATFORMS.iter().for_each(|(pos, size)| spawn_platform(commands, *pos, *size));
}

fn spawn_player_ui(commands: &mut Commands) {
	commands.spawn((
		Text::new("Health: 100/100 | Level: 1"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(10.0),
			left: Val::Px(10.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: 20.0,
			..default()
		},
		PlayerStatsText,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(40.0),
			left: Val::Px(10.0),
			width: Val::Px(300.0),
			height: Val::Px(20.0),
			..default()
		},
		BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
		XPBarBackground,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(40.0),
			left: Val::Px(10.0),
			width: Val::Px(0.0),
			height: Val::Px(20.0),
			..default()
		},
		BackgroundColor(Color::srgb(0.2, 0.6, 0.9)),
		XPBarForeground,
	));

	commands.spawn((
		Text::new("XP: 0/100"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(42.0),
			left: Val::Px(15.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: 16.0,
			..default()
		},
		XPText,
	));
}

fn spawn_player(
	mut commands: Commands,
	game_config: Option<Res<crate::GameConfig>>,
	config_assets: Res<Assets<crate::GameConfigData>>,
	player_query: Query<(), With<Player>>,
) {
	// Only spawn once
	if !player_query.is_empty() {
		return;
	}

	const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 40.0);
	const PLAYER_SPAWN: Vec3 = Vec3::new(0.0, -200.0, 0.0);

	let Some(game_config) = game_config else {
		return;
	};

	let Some(config_data) = config_assets.get(&game_config.config_handle) else {
		return;
	};

	commands.spawn((
		Sprite {
			color: Color::srgb(0.2, 0.4, 0.9),
			custom_size: Some(PLAYER_SIZE),
			..default()
		},
		Transform::from_translation(PLAYER_SPAWN),
		Player::default(),
		crate::behaviors::PlayerTag,
		crate::behaviors::Damageable {
			health: 100.0,
			max_health: 100.0,
		},
		crate::physics::Velocity { x: 0.0, y: 0.0 },
		crate::physics::Grounded(false),
		NeedsInitialWeapon {
			weapon_id: config_data.initial_weapon.weapon_id.clone(),
			count: config_data.initial_weapon.level,
		},
	));

	spawn_platforms(&mut commands);
	spawn_player_ui(&mut commands);
}

fn spawn_initial_weapon(
	mut commands: Commands,
	mut player_query: Query<(Entity, &NeedsInitialWeapon)>,
	weapon_registry: Option<Res<crate::weapons::WeaponRegistry>>,
	weapon_data_assets: Res<Assets<crate::weapons::WeaponData>>,
) {
	let Some(registry) = weapon_registry else { return };

	for (entity, needs_weapon) in player_query.iter_mut() {
		if let Some(handle) = registry.get(&needs_weapon.weapon_id) {
			if let Some(weapon_data) = weapon_data_assets.get(handle) {
				crate::weapons::spawn_entity_from_data(
					&mut commands,
					weapon_data,
					needs_weapon.count,
				);
				commands.entity(entity).remove::<NeedsInitialWeapon>();
			}
		}
	}
}

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut crate::physics::Velocity, &Player)>,
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
                if axis_value.abs() > 0.1 {  // Deadzone
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

        velocity.x = direction * player.speed;
    }
}

fn player_jump(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut crate::physics::Velocity, &Player, &crate::physics::Grounded)>,
) {
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
    player_query: Query<(&Player, &crate::behaviors::Damageable), Or<(Changed<Player>, Changed<crate::behaviors::Damageable>)>>,
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
    if player_xp.is_changed() {
        // Update XP bar width
        if let Ok(mut node) = xp_bar_query.single_mut() {
            let xp_percent = (player_xp.current_xp as f32 / player_xp.xp_to_next_level as f32).clamp(0.0, 1.0);
            node.width = Val::Px(300.0 * xp_percent);
        }

        // Update XP text
        if let Ok(mut text) = xp_text_query.single_mut() {
            **text = format!("XP: {}/{}", player_xp.current_xp, player_xp.xp_to_next_level);
        }
    }
}
