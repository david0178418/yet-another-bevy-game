use bevy::prelude::*;
use rand::seq::SliceRandom;

pub struct PowerupsPlugin;

fn get_powerup_name(
	powerup: &crate::PowerupDefinition,
	weapon_registry: Option<&crate::weapons::WeaponRegistry>,
	weapon_assets: &Assets<crate::weapons::WeaponData>,
) -> String {
	match powerup {
		crate::PowerupDefinition::Weapon(id) => {
			weapon_registry
				.and_then(|r| r.get(id))
				.and_then(|h| weapon_assets.get(h))
				.map(|w| w.name.clone())
				.unwrap_or_else(|| id.clone())
		}
		crate::PowerupDefinition::StatBoost(data) => data.name.clone(),
	}
}

fn get_powerup_description(
	powerup: &crate::PowerupDefinition,
	weapon_registry: Option<&crate::weapons::WeaponRegistry>,
	weapon_assets: &Assets<crate::weapons::WeaponData>,
) -> String {
	match powerup {
		crate::PowerupDefinition::Weapon(id) => {
			weapon_registry
				.and_then(|r| r.get(id))
				.and_then(|h| weapon_assets.get(h))
				.map(|w| w.description.clone())
				.unwrap_or_else(|| format!("Unknown weapon: {}", id))
		}
		crate::PowerupDefinition::StatBoost(data) => data.description.clone(),
	}
}

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
    pub options: Vec<crate::PowerupDefinition>,
    pub selected_index: usize,
}

#[derive(Component)]
struct PowerupButton {
    powerup_def: crate::PowerupDefinition,
    index: usize,
}

fn apply_powerup(
	powerup_def: &crate::PowerupDefinition,
	commands: &mut Commands,
	player: &mut crate::player::Player,
	player_damageable: &mut crate::behaviors::Damageable,
	weapon_registry: Option<&crate::weapons::WeaponRegistry>,
	weapon_data_assets: &Assets<crate::weapons::WeaponData>,
) {
	match powerup_def {
		crate::PowerupDefinition::Weapon(weapon_id) => {
			if let Some(registry) = weapon_registry {
				if let Some(handle) = registry.get(weapon_id) {
					if let Some(weapon_data) = weapon_data_assets.get(handle) {
						crate::weapons::spawn_entity_from_data(commands, weapon_data, 1);
					}
				}
			}
		}
		crate::PowerupDefinition::StatBoost(boost) => {
			match boost.stat {
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
			}
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
		commands.entity(entity).despawn();
	}
	powerup_state.showing = false;
	powerup_state.options.clear();
	time.unpause();
}

fn handle_level_up(
    mut commands: Commands,
    mut level_up_events: MessageReader<crate::experience::LevelUpEvent>,
    mut powerup_state: ResMut<PowerupState>,
    mut time: ResMut<Time<Virtual>>,
    game_config: Option<Res<crate::GameConfig>>,
    config_assets: Res<Assets<crate::GameConfigData>>,
    weapon_registry: Option<Res<crate::weapons::WeaponRegistry>>,
    weapon_data_assets: Res<Assets<crate::weapons::WeaponData>>,
) {
    for _ in level_up_events.read() {
        if powerup_state.showing {
            continue;
        }

        let Some(game_config) = game_config.as_ref() else {
            continue;
        };

        let Some(config_data) = config_assets.get(&game_config.config_handle) else {
            continue;
        };

        // Generate random powerup options from the pool
        let mut rng = rand::thread_rng();
        let options: Vec<crate::PowerupDefinition> = config_data.powerup_pool
            .choose_multiple(&mut rng, crate::constants::POWERUP_OPTIONS_COUNT)
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, crate::constants::POWERUP_OVERLAY_ALPHA)),
            PowerupUIContainer,
        )).id();

        let button_container = commands.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(crate::constants::POWERUP_BUTTON_GAP),
            ..default()
        }).id();

        commands.entity(container).add_child(button_container);

        // Title
        let title = commands.spawn((
            Text::new("LEVEL UP! Choose a Powerup:"),
            TextFont {
                font_size: crate::constants::UI_FONT_SIZE_LARGE,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.3)),
            Node {
                margin: UiRect::bottom(Val::Px(crate::constants::POWERUP_TITLE_MARGIN)),
                ..default()
            },
        )).id();

        commands.entity(button_container).add_child(title);

        // Create buttons for each option
        for (index, powerup) in options.iter().enumerate() {
            // First button is selected by default
            let bg_color = if index == 0 {
                crate::constants::POWERUP_COLOR_SELECTED
            } else {
                crate::constants::POWERUP_COLOR_NORMAL
            };

            let button = commands.spawn((
                Button,
                Node {
                    width: Val::Px(crate::constants::POWERUP_BUTTON_WIDTH),
                    height: Val::Px(crate::constants::POWERUP_BUTTON_HEIGHT),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(crate::constants::POWERUP_BUTTON_PADDING)),
                    ..default()
                },
                BackgroundColor(bg_color),
                PowerupButton {
                    powerup_def: powerup.clone(),
                    index,
                },
            )).id();

            let text_container = commands.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            }).id();

            let name_text = commands.spawn((
                Text::new(get_powerup_name(powerup, weapon_registry.as_deref(), &weapon_data_assets)),
                TextFont {
                    font_size: crate::constants::UI_FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(Color::WHITE),
            )).id();

            let desc_text = commands.spawn((
                Text::new(get_powerup_description(powerup, weapon_registry.as_deref(), &weapon_data_assets)),
                TextFont {
                    font_size: crate::constants::UI_FONT_SIZE_SMALL,
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
    mut player_query: Query<(&mut crate::player::Player, &mut crate::behaviors::Damageable), With<crate::behaviors::PlayerTag>>,
    mut time: ResMut<Time<Virtual>>,
    gamepads: Query<&Gamepad>,
    weapon_registry: Option<Res<crate::weapons::WeaponRegistry>>,
    weapon_data_assets: Res<Assets<crate::weapons::WeaponData>>,
) {
    for (button, interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Ok((mut player, mut damageable)) = player_query.single_mut() {
                    apply_powerup(&button.powerup_def, &mut commands, &mut player, &mut damageable, weapon_registry.as_deref(), &weapon_data_assets);
                }
                cleanup_powerup_ui(&mut commands, &ui_query, &mut powerup_state, &mut time);
            }
            Interaction::Hovered => {
                *bg_color = crate::constants::POWERUP_COLOR_HOVERED.into();
            }
            Interaction::None => {
                *bg_color = crate::constants::POWERUP_COLOR_NORMAL.into();
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
                    if let Ok((mut player, mut damageable)) = player_query.single_mut() {
                        apply_powerup(&button.powerup_def, &mut commands, &mut player, &mut damageable, weapon_registry.as_deref(), &weapon_data_assets);
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
                *bg_color = crate::constants::POWERUP_COLOR_SELECTED.into();
            } else {
                *bg_color = crate::constants::POWERUP_COLOR_NORMAL.into();
            }
        }
    }
}
