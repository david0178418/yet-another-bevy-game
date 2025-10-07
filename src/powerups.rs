use bevy::prelude::*;
use rand::seq::SliceRandom;

pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PowerupState {
            showing: false,
            options: vec![],
        })
        .add_systems(Update, (
            handle_level_up,
            handle_powerup_selection,
        ));
    }
}

#[derive(Resource)]
pub struct PowerupState {
    pub showing: bool,
    pub options: Vec<PowerupType>,
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
        for powerup in options.iter() {
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
                BackgroundColor(Color::srgb(0.2, 0.2, 0.3)),
                PowerupButton {
                    powerup_type: powerup.clone(),
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
        (&Interaction, &PowerupButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut powerup_state: ResMut<PowerupState>,
    ui_query: Query<Entity, With<PowerupUIContainer>>,
    mut player_query: Query<(Entity, &mut crate::player::Player)>,
    mut time: ResMut<Time<Virtual>>,
) {
    for (interaction, button, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Apply powerup
                if let Ok((player_entity, mut player)) = player_query.get_single_mut() {
                    match button.powerup_type {
                        PowerupType::OrbitingBlade => {
                            crate::weapons::spawn_orbiting_blade(&mut commands, player_entity, 1);
                        }
                        PowerupType::AutoShooter => {
                            crate::weapons::spawn_auto_shooter(&mut commands, player_entity);
                        }
                        PowerupType::SpeedBoost => {
                            player.speed += 50.0;
                        }
                        PowerupType::JumpBoost => {
                            player.jump_force += 100.0;
                        }
                        PowerupType::MaxHealthIncrease => {
                            player.max_health += 20.0;
                            player.health = player.max_health; // Full heal
                        }
                    }
                }

                // Remove UI
                for entity in ui_query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                powerup_state.showing = false;
                powerup_state.options.clear();

                // Resume the game
                time.unpause();
            }
            Interaction::Hovered => {
                *bg_color = Color::srgb(0.3, 0.3, 0.4).into();
            }
            Interaction::None => {
                *bg_color = Color::srgb(0.2, 0.2, 0.3).into();
            }
        }
    }
}
