use bevy::prelude::*;
use rand::seq::SliceRandom;

pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PowerupState {
            showing: false,
            options: vec![],
            selected_index: 0,
        })
        .add_systems(Update, (
            handle_level_up,
            handle_powerup_navigation,
            handle_powerup_selection,
        ));
    }
}

#[derive(Resource)]
pub struct PowerupState {
    pub showing: bool,
    pub options: Vec<PowerupType>,
    pub selected_index: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PowerupType {
    OrbitingBlade,
    AutoShooter,
    SpeedBoost,
    JumpBoost,
    MaxHealthIncrease,
}

impl PowerupType {
    pub fn name(&self) -> &str {
        match self {
            PowerupType::OrbitingBlade => "Orbiting Blade",
            PowerupType::AutoShooter => "Auto Shooter",
            PowerupType::SpeedBoost => "Speed Boost",
            PowerupType::JumpBoost => "Jump Boost",
            PowerupType::MaxHealthIncrease => "Max Health +20",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            PowerupType::OrbitingBlade => "Adds a blade that orbits around you",
            PowerupType::AutoShooter => "Auto-fires projectiles at enemies",
            PowerupType::SpeedBoost => "Increases movement speed",
            PowerupType::JumpBoost => "Increases jump height",
            PowerupType::MaxHealthIncrease => "Increases maximum health and heals",
        }
    }
}

#[derive(Component)]
struct PowerupButton {
    powerup_type: PowerupType,
    index: usize,
}

fn apply_powerup(
	powerup_type: &PowerupType,
	commands: &mut Commands,
	player: &mut crate::player::Player,
	blade_query: &mut Query<&mut crate::weapons::OrbitingBlade>,
	weapon_defs: Option<&crate::weapons::WeaponDefinitions>,
	weapon_data_assets: &Assets<crate::weapons::WeaponData>,
) {
	match powerup_type {
		PowerupType::OrbitingBlade => {
			crate::weapons::spawn_orbiting_blade(commands, 1, blade_query, weapon_defs, weapon_data_assets);
		}
		PowerupType::AutoShooter => {
			crate::weapons::spawn_auto_shooter(commands, weapon_defs, weapon_data_assets);
		}
		PowerupType::SpeedBoost => {
			player.speed += 50.0;
		}
		PowerupType::JumpBoost => {
			player.jump_force += 100.0;
		}
		PowerupType::MaxHealthIncrease => {
			player.max_health += 20.0;
			player.health = player.max_health;
		}
	}
}

fn cleanup_powerup_ui(
	commands: &mut Commands,
	ui_query: &Query<Entity, With<PowerupUIContainer>>,
	powerup_state: &mut PowerupState,
	time: &mut Time<Virtual>,
) {
	for entity in ui_query.iter() {
		commands.entity(entity).despawn_recursive();
	}
	powerup_state.showing = false;
	powerup_state.options.clear();
	time.unpause();
}

fn handle_level_up(
    mut commands: Commands,
    mut level_up_events: EventReader<crate::experience::LevelUpEvent>,
    mut powerup_state: ResMut<PowerupState>,
    mut time: ResMut<Time<Virtual>>,
) {
    for _ in level_up_events.read() {
        if powerup_state.showing {
            continue;
        }

        // Generate 3 random powerup options
        let all_powerups = vec![
            PowerupType::OrbitingBlade,
            PowerupType::AutoShooter,
            PowerupType::SpeedBoost,
            PowerupType::JumpBoost,
            PowerupType::MaxHealthIncrease,
        ];

        let mut rng = rand::thread_rng();
        let options: Vec<PowerupType> = all_powerups
            .choose_multiple(&mut rng, 3)
            .cloned()
            .collect();

        powerup_state.showing = true;
        powerup_state.options = options.clone();
        powerup_state.selected_index = 0;

        // Pause the game
        time.pause();

        // Create UI overlay
        let container = commands.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
            PowerupUIContainer,
        )).id();

        let button_container = commands.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(20.0),
            ..default()
        }).id();

        commands.entity(container).add_child(button_container);

        // Title
        let title = commands.spawn((
            Text::new("LEVEL UP! Choose a Powerup:"),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.3)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        )).id();

        commands.entity(button_container).add_child(title);

        // Create buttons for each option
        for (index, powerup) in options.iter().enumerate() {
            // First button is selected by default
            let bg_color = if index == 0 {
                Color::srgb(0.3, 0.3, 0.5) // Highlighted
            } else {
                Color::srgb(0.2, 0.2, 0.3) // Normal
            };

            let button = commands.spawn((
                Button,
                Node {
                    width: Val::Px(400.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(bg_color),
                PowerupButton {
                    powerup_type: powerup.clone(),
                    index,
                },
            )).id();

            let text_container = commands.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            }).id();

            let name_text = commands.spawn((
                Text::new(powerup.name()),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            )).id();

            let desc_text = commands.spawn((
                Text::new(powerup.description()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            )).id();

            commands.entity(text_container).add_child(name_text);
            commands.entity(text_container).add_child(desc_text);
            commands.entity(button).add_child(text_container);
            commands.entity(button_container).add_child(button);
        }
    }
}

#[derive(Component)]
struct PowerupUIContainer;

fn handle_powerup_selection(
    mut commands: Commands,
    mut interaction_query: Query<
        (&PowerupButton, &Interaction, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut powerup_state: ResMut<PowerupState>,
    ui_query: Query<Entity, With<PowerupUIContainer>>,
    mut player_query: Query<(Entity, &mut crate::player::Player)>,
    mut time: ResMut<Time<Virtual>>,
    gamepads: Query<&Gamepad>,
    mut blade_query: Query<&mut crate::weapons::OrbitingBlade>,
    weapon_defs: Option<Res<crate::weapons::WeaponDefinitions>>,
    weapon_data_assets: Res<Assets<crate::weapons::WeaponData>>,
) {
    for (button, interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Ok((_, mut player)) = player_query.get_single_mut() {
                    apply_powerup(&button.powerup_type, &mut commands, &mut player, &mut blade_query, weapon_defs.as_deref(), &weapon_data_assets);
                }
                cleanup_powerup_ui(&mut commands, &ui_query, &mut powerup_state, &mut time);
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.3, 0.3, 0.4).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.2, 0.2, 0.3).into();
            }
        }
    }

    // Gamepad selection
    if !powerup_state.showing {
        return;
    }

    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::South) {
            for (button, _, _) in interaction_query.iter() {
                if button.index == powerup_state.selected_index {
                    if let Ok((_, mut player)) = player_query.get_single_mut() {
                        apply_powerup(&button.powerup_type, &mut commands, &mut player, &mut blade_query, weapon_defs.as_deref(), &weapon_data_assets);
                    }
                    cleanup_powerup_ui(&mut commands, &ui_query, &mut powerup_state, &mut time);
                    break;
                }
            }
        }
    }
}

fn handle_powerup_navigation(
    mut powerup_state: ResMut<PowerupState>,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut button_query: Query<(&PowerupButton, &mut BackgroundColor)>,
) {
    if !powerup_state.showing || powerup_state.options.is_empty() {
        return;
    }

    let mut direction = 0i32;

    // Keyboard navigation
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        direction = -1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        direction = 1;
    }

    // Gamepad navigation
    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            direction = -1;
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            direction = 1;
        }
    }

    if direction != 0 {
        let num_options = powerup_state.options.len();
        if direction < 0 {
            powerup_state.selected_index = if powerup_state.selected_index == 0 {
                num_options - 1
            } else {
                powerup_state.selected_index - 1
            };
        } else {
            powerup_state.selected_index = (powerup_state.selected_index + 1) % num_options;
        }

        // Update button colors based on selection
        for (button, mut bg_color) in button_query.iter_mut() {
            if button.index == powerup_state.selected_index {
                *bg_color = Color::srgb(0.3, 0.3, 0.5).into(); // Highlighted
            } else {
                *bg_color = Color::srgb(0.2, 0.2, 0.3).into(); // Normal
            }
        }
    }
}
