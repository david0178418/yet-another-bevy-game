use bevy::{prelude::*, ecs::system::SystemParam};
use rand::seq::SliceRandom;

pub struct PowerupsPlugin;

#[derive(SystemParam)]
struct WeaponResources<'w> {
	registry: Option<Res<'w, crate::weapons::WeaponRegistry>>,
	assets: Res<'w, Assets<crate::weapons::WeaponData>>,
}

#[derive(SystemParam)]
struct PowerupUIState<'w, 's> {
	state: ResMut<'w, PowerupState>,
	ui_query: Query<'w, 's, Entity, With<PowerupUIContainer>>,
	time: ResMut<'w, Time<Virtual>>,
}

fn get_powerup_name(
	powerup: &crate::PowerupDefinition,
	weapon_resources: &WeaponResources,
) -> String {
	match powerup {
		crate::PowerupDefinition::Weapon(id) => {
			weapon_resources.registry
				.as_ref()
				.and_then(|r| r.get(id))
				.and_then(|h| weapon_resources.assets.get(h))
				.map(|w| w.name.clone())
				.unwrap_or_else(|| id.clone())
		}
		crate::PowerupDefinition::StatBoost(data) => data.name.clone(),
	}
}

fn get_powerup_description(
	powerup: &crate::PowerupDefinition,
	weapon_resources: &WeaponResources,
) -> String {
	match powerup {
		crate::PowerupDefinition::Weapon(id) => {
			weapon_resources.registry
				.as_ref()
				.and_then(|r| r.get(id))
				.and_then(|h| weapon_resources.assets.get(h))
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
	weapon_resources: &WeaponResources,
) {
	match powerup_def {
		crate::PowerupDefinition::Weapon(weapon_id) => {
			if let Some(registry) = weapon_resources.registry.as_ref() {
				if let Some(handle) = registry.get(weapon_id) {
					if let Some(weapon_data) = weapon_resources.assets.get(handle) {
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
	ui_state: &mut PowerupUIState,
) {
	for entity in ui_state.ui_query.iter() {
		commands.entity(entity).despawn();
	}
	ui_state.state.showing = false;
	ui_state.state.options.clear();
	ui_state.time.unpause();
}

fn handle_level_up(
    mut commands: Commands,
    mut level_up_events: MessageReader<crate::experience::LevelUpEvent>,
    mut powerup_state: ResMut<PowerupState>,
    mut time: ResMut<Time<Virtual>>,
    game_config: Option<Res<crate::GameConfig>>,
    config_assets: Res<Assets<crate::GameConfigData>>,
    weapon_resources: WeaponResources,
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
                Text::new(get_powerup_name(powerup, &weapon_resources)),
                TextFont {
                    font_size: crate::constants::UI_FONT_SIZE_MEDIUM,
                    ..default()
                },
                TextColor(Color::WHITE),
            )).id();

            let desc_text = commands.spawn((
                Text::new(get_powerup_description(powerup, &weapon_resources)),
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
    mut ui_state: PowerupUIState,
    mut player_query: Query<(&mut crate::player::Player, &mut crate::behaviors::Damageable), With<crate::behaviors::PlayerTag>>,
    gamepads: Query<&Gamepad>,
    weapon_resources: WeaponResources,
) {
    for (button, interaction, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if let Ok((mut player, mut damageable)) = player_query.single_mut() {
                    apply_powerup(&button.powerup_def, &mut commands, &mut player, &mut damageable, &weapon_resources);
                }
                cleanup_powerup_ui(&mut commands, &mut ui_state);
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
    if !ui_state.state.showing {
        return;
    }

    for gamepad in gamepads.iter() {
        if gamepad.just_pressed(GamepadButton::South) {
            for (button, _, _) in interaction_query.iter() {
                if button.index == ui_state.state.selected_index {
                    if let Ok((mut player, mut damageable)) = player_query.single_mut() {
                        apply_powerup(&button.powerup_def, &mut commands, &mut player, &mut damageable, &weapon_resources);
                    }
                    cleanup_powerup_ui(&mut commands, &mut ui_state);
                    break;
                }
            }
        }
    }
}

fn handle_powerup_navigation(
    mut ui_state: PowerupUIState,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut button_query: Query<(&PowerupButton, &mut BackgroundColor)>,
) {
    if !ui_state.state.showing || ui_state.state.options.is_empty() {
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
        let num_options = ui_state.state.options.len();
        if direction < 0 {
            ui_state.state.selected_index = if ui_state.state.selected_index == 0 {
                num_options - 1
            } else {
                ui_state.state.selected_index - 1
            };
        } else {
            ui_state.state.selected_index = (ui_state.state.selected_index + 1) % num_options;
        }

        // Update button colors based on selection
        for (button, mut bg_color) in button_query.iter_mut() {
            if button.index == ui_state.state.selected_index {
                *bg_color = crate::constants::POWERUP_COLOR_SELECTED.into();
            } else {
                *bg_color = crate::constants::POWERUP_COLOR_NORMAL.into();
            }
        }
    }
}
