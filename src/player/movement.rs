use bevy::prelude::*;

pub fn player_movement(
	keyboard: Res<ButtonInput<KeyCode>>,
	gamepads: Query<&Gamepad>,
	mut query: Query<(&mut crate::physics::Velocity, &super::Player), Without<crate::behaviors::EnergyCharging>>,
	time: Res<Time>, // Use real time for input, not virtual (paused) time
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
				if axis_value.abs() > crate::constants::GAMEPAD_DEADZONE {
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

		// Acceleration-based movement
		let target_speed = direction * player.speed;
		let speed_diff = target_speed - velocity.x;

		if speed_diff.abs() > 0.01 {
			// Choose acceleration or deceleration based on input
			let accel = if direction.abs() > 0.01 {
				crate::constants::PLAYER_ACCELERATION
			} else {
				crate::constants::PLAYER_DECELERATION
			};

			let change = speed_diff.signum() * accel * time.delta_secs();

			// Snap to target if close enough, otherwise apply acceleration
			if speed_diff.abs() <= change.abs() {
				velocity.x = target_speed;
			} else {
				velocity.x += change;
			}
		}
	}
}

pub fn player_jump(
	keyboard: Res<ButtonInput<KeyCode>>,
	gamepads: Query<&Gamepad>,
	mut query: Query<
		(
			&mut crate::physics::Velocity,
			&super::Player,
			&crate::physics::Grounded,
		),
		Without<crate::behaviors::EnergyCharging>,
	>,
	powerup_state: Res<crate::powerups::PowerupState>,
) {
	// Don't process jump input while menu is showing
	if powerup_state.showing {
		return;
	}

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
