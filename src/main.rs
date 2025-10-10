use bevy::{prelude::*, ui::UiScale, window::{WindowResized, WindowResolution}, asset::AssetLoader, camera::{Viewport, ScalingMode}};
use serde::Deserialize;

mod player;
mod physics;
mod enemy;
mod weapons;
mod experience;
mod powerups;
mod combat;
mod behaviors;
mod constants;

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

#[derive(Deserialize, Clone)]
pub struct InitialWeapon {
	pub weapon_id: String,
	pub level: u32,
}

#[derive(Deserialize, Clone)]
pub enum StatType {
	Speed,
	JumpForce,
	MaxHealth,
}

#[derive(Deserialize, Clone)]
pub struct StatBoostData {
	pub stat: StatType,
	pub value: f32,
	pub name: String,
	pub description: String,
}

#[derive(Deserialize, Clone)]
pub enum PowerupDefinition {
	Weapon(String),
	StatBoost(StatBoostData),
}


#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct GameConfigData {
	pub weapon_ids: Vec<String>,
	pub enemy_ids: Vec<String>,
	pub initial_weapon: InitialWeapon,
	pub powerup_pool: Vec<PowerupDefinition>,
}

#[derive(Default)]
struct GameConfigLoader;

impl AssetLoader for GameConfigLoader {
	type Asset = GameConfigData;
	type Settings = ();
	type Error = std::io::Error;

	async fn load(
		&self,
		reader: &mut dyn bevy::asset::io::Reader,
		_settings: &Self::Settings,
		_load_context: &mut bevy::asset::LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = Vec::new();
		reader.read_to_end(&mut bytes).await?;
		let data = ron::de::from_bytes::<GameConfigData>(&bytes)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		Ok(data)
	}

	fn extensions(&self) -> &[&str] {
		&["game_config.ron"]
	}
}

#[derive(Resource)]
pub struct GameConfig {
	pub config_handle: Handle<GameConfigData>,
}

fn main() {
	App::new()
		.add_plugins(DefaultPlugins.set(WindowPlugin {
			primary_window: Some(Window {
				title: "Vampire Survivors Platformer".to_string(),
				resolution: WindowResolution::new(1280, 720),
				resizable: true,
				..default()
			}),
			..default()
		}))
		.init_asset::<GameConfigData>()
		.init_asset_loader::<GameConfigLoader>()
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
		.add_systems(Startup, (setup_camera, load_game_config))
		.add_systems(Update, update_camera_viewport)
		.run();
}

fn load_game_config(mut commands: Commands, asset_server: Res<AssetServer>) {
	let config_handle = asset_server.load("game_config.ron");
	commands.insert_resource(GameConfig { config_handle });
}

fn setup_camera(mut commands: Commands, windows: Query<&Window>, mut ui_scale: ResMut<UiScale>) {
	if let Ok(window) = windows.single() {
		let viewport = calculate_viewport(window.width(), window.height());
		let scale = calculate_ui_scale(window.width(), window.height());
		ui_scale.0 = scale;

		commands.spawn((
			Camera2d,
			Camera {
				viewport: Some(viewport),
				..default()
			},
			Projection::from(OrthographicProjection {
				scaling_mode: ScalingMode::Fixed {
					width: GAME_WIDTH,
					height: GAME_HEIGHT,
				},
				..OrthographicProjection::default_2d()
			}),
			GameCamera,
		));
	}
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

fn calculate_ui_scale(window_width: f32, window_height: f32) -> f32 {
	let window_aspect = window_width / window_height;

	if window_aspect > ASPECT_RATIO {
		window_height / GAME_HEIGHT
	} else {
		window_width / GAME_WIDTH
	}
}

fn update_camera_viewport(
	mut resize_events: MessageReader<WindowResized>,
	mut camera_query: Query<&mut Camera, With<GameCamera>>,
	mut ui_scale: ResMut<UiScale>,
) {
	for event in resize_events.read() {
		if let Ok(mut camera) = camera_query.single_mut() {
			camera.viewport = Some(calculate_viewport(event.width, event.height));
			ui_scale.0 = calculate_ui_scale(event.width, event.height);
		}
	}
}
