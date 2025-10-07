use bevy::prelude::*;

mod player;
mod physics;
mod enemy;
mod weapons;
mod experience;
mod powerups;
mod combat;

use player::PlayerPlugin;
use physics::PhysicsPlugin;
use enemy::EnemyPlugin;
use weapons::WeaponsPlugin;
use experience::ExperiencePlugin;
use powerups::PowerupsPlugin;
use combat::CombatPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vampire Survivors Platformer".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            PhysicsPlugin,
            PlayerPlugin,
            EnemyPlugin,
            WeaponsPlugin,
            ExperiencePlugin,
            PowerupsPlugin,
            CombatPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.15)))
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}
