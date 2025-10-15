use bevy::{ecs::system::SystemParam, prelude::*};

pub mod application;
pub mod ui;

pub struct PowerupsPlugin;

#[derive(SystemParam)]
pub struct WeaponResources<'w> {
	pub registry: Option<Res<'w, crate::weapons::WeaponRegistry>>,
	pub assets: Res<'w, Assets<crate::weapons::WeaponData>>,
}

impl Plugin for PowerupsPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(PowerupState {
			showing: false,
			options: vec![],
			selected_index: 0,
		})
		.add_systems(
			Update,
			(
				ui::handle_level_up,
				ui::handle_powerup_navigation,
				ui::handle_powerup_selection,
			),
		);
	}
}

#[derive(Resource)]
pub struct PowerupState {
	pub showing: bool,
	pub options: Vec<crate::PowerupDefinition>,
	pub selected_index: usize,
}
