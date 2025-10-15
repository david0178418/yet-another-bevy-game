use bevy::prelude::*;

mod energy;
mod movement;
mod spawning;
mod ui;

// Re-export public items if needed in the future
// pub use energy::RepulsionFieldIndicator;
// pub use spawning::NeedsInitialWeapons;
// pub use ui::{
// 	EnergyBarBackground, EnergyBarForeground, EnergyText, PlayerStatsText, XPBarBackground,
// 	XPBarForeground, XPText,
// };

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				// Process input before physics for minimal latency
				movement::player_movement,
				movement::player_jump,
				energy::handle_energy_charging_input,
			)
				.before(crate::physics::PhysicsSet),
		)
		.add_systems(
			Update,
			(
				spawning::spawn_player,
				spawning::spawn_initial_weapon,
				ui::update_player_stats_display,
				ui::update_xp_bar,
				energy::regenerate_energy,
				energy::charge_energy,
				ui::update_energy_bar,
			),
		)
		.add_systems(
			Update,
			(energy::apply_repulsion_field, energy::cleanup_repulsion_markers)
				.chain()
				.before(crate::movement::MovementSystemSet),
		);
	}
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
