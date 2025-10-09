use bevy::{prelude::*, render::camera::{ScalingMode, Viewport}, window::WindowResized};

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

const GAME_WIDTH: f32 = 1280.0;
const GAME_HEIGHT: f32 = 720.0;
const ASPECT_RATIO: f32 = GAME_WIDTH / GAME_HEIGHT;

#[derive(Component)]
struct GameCamera;

#[derive(Resource)]
pub struct GameConfig {
	pub initial_weapon_level: u32,
}

fn parse_cli_args() -> GameConfig {
	let args: Vec<String> = std::env::args().collect();
	let mut initial_weapon_level = 1;

	let mut i = 1;
	while i < args.len() {
		match args[i].as_str() {
			"--initial-weapon-level" | "-w" => {
				if i + 1 < args.len() {
					initial_weapon_level = args[i + 1].parse().unwrap_or(1);
					i += 2;
				} else {
					i += 1;
				}
			}
			_ => {
				i += 1;
			}
		}
	}

	GameConfig {
		initial_weapon_level,
	}
}

fn main() {
	let config = parse_cli_args();
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vampire Survivors Platformer".to_string(),
                resolution: (1280.0, 720.0).into(),
                resizable: true,
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
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(config)
        .add_systems(Startup, setup_camera)
        .add_systems(Update, update_camera_viewport)
        .run();
}

fn setup_camera(mut commands: Commands, windows: Query<&Window>) {
    let window = windows.single();
    let viewport = calculate_viewport(window.width(), window.height());

    commands.spawn((
        Camera2d,
        Camera {
            viewport: Some(viewport),
            ..default()
        },
        OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: GAME_WIDTH,
                height: GAME_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        },
        GameCamera,
    ));
}

fn calculate_viewport(window_width: f32, window_height: f32) -> Viewport {
    let window_aspect = window_width / window_height;

    let (viewport_width, viewport_height) = if window_aspect > ASPECT_RATIO {
        let height = window_height;
        let width = height * ASPECT_RATIO;
        (width, height)
    } else {
        let width = window_width;
        let height = width / ASPECT_RATIO;
        (width, height)
    };

    let x = (window_width - viewport_width) / 2.0;
    let y = (window_height - viewport_height) / 2.0;

    Viewport {
        physical_position: UVec2::new(x as u32, y as u32),
        physical_size: UVec2::new(viewport_width as u32, viewport_height as u32),
        ..default()
    }
}

fn update_camera_viewport(
    mut resize_events: EventReader<WindowResized>,
    mut camera_query: Query<&mut Camera, With<GameCamera>>,
) {
    for event in resize_events.read() {
        if let Ok(mut camera) = camera_query.get_single_mut() {
            camera.viewport = Some(calculate_viewport(event.width, event.height));
        }
    }
}
