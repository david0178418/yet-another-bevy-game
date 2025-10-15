use bevy::prelude::*;

type PlayerStatsQuery<'w, 's> = Query<
	'w,
	's,
	(&'static super::Player, &'static crate::behaviors::Damageable),
	Or<(Changed<super::Player>, Changed<crate::behaviors::Damageable>)>,
>;

#[derive(Component)]
pub struct PlayerStatsText;

#[derive(Component)]
pub struct XPBarBackground;

#[derive(Component)]
pub struct XPBarForeground;

#[derive(Component)]
pub struct XPText;

#[derive(Component)]
pub struct EnergyBarBackground;

#[derive(Component)]
pub struct EnergyBarForeground;

#[derive(Component)]
pub struct EnergyText;

pub fn spawn_player_ui(commands: &mut Commands) {
	use crate::constants::*;

	commands.spawn((
		Text::new("Health: 100/100 | Level: 1"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(UI_MARGIN),
			left: Val::Px(UI_MARGIN),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_NORMAL,
			..default()
		},
		PlayerStatsText,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(XP_BAR_WIDTH),
			height: Val::Px(XP_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(XP_BAR_COLOR_BG),
		ZIndex(0),
		XPBarBackground,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(0.0),
			height: Val::Px(XP_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(XP_BAR_COLOR_FG),
		ZIndex(1),
		XPBarForeground,
	));

	commands.spawn((
		Text::new("XP: 0/100"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(XP_BAR_TOP + 2.0),
			left: Val::Px(UI_MARGIN + 5.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_SMALL,
			..default()
		},
		ZIndex(2),
		XPText,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(ENERGY_BAR_WIDTH),
			height: Val::Px(ENERGY_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(ENERGY_BAR_COLOR_BG),
		ZIndex(0),
		EnergyBarBackground,
	));

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP),
			left: Val::Px(UI_MARGIN),
			width: Val::Px(ENERGY_BAR_WIDTH),
			height: Val::Px(ENERGY_BAR_HEIGHT),
			..default()
		},
		BackgroundColor(ENERGY_BAR_COLOR_FG),
		ZIndex(1),
		EnergyBarForeground,
	));

	commands.spawn((
		Text::new("Energy: 100/100"),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(ENERGY_BAR_TOP + 2.0),
			left: Val::Px(UI_MARGIN + 5.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: UI_FONT_SIZE_SMALL,
			..default()
		},
		ZIndex(2),
		EnergyText,
	));
}

pub fn update_player_stats_display(
	player_query: PlayerStatsQuery,
	mut text_query: Query<&mut Text, With<PlayerStatsText>>,
) {
	if let Ok((player, damageable)) = player_query.single() {
		if let Ok(mut text) = text_query.single_mut() {
			**text = format!(
				"Health: {:.0}/{:.0} | Level: {}",
				damageable.health, damageable.max_health, player.level
			);
		}
	}
}

pub fn update_xp_bar(
	player_xp: Res<crate::experience::PlayerExperience>,
	mut xp_bar_query: Query<&mut Node, With<XPBarForeground>>,
	mut xp_text_query: Query<&mut Text, With<XPText>>,
) {
	// Only update if UI exists
	let Ok(mut node) = xp_bar_query.single_mut() else {
		return;
	};

	let Ok(mut text) = xp_text_query.single_mut() else {
		return;
	};

	// Update XP bar width
	let xp_percent =
		(player_xp.current_xp as f32 / player_xp.xp_to_next_level as f32).clamp(0.0, 1.0);
	let new_width = crate::constants::XP_BAR_WIDTH * xp_percent;
	node.width = Val::Px(new_width);

	// Update XP text
	**text = format!(
		"XP: {}/{}",
		player_xp.current_xp, player_xp.xp_to_next_level
	);
}

pub fn update_energy_bar(
	player_query: Query<&crate::behaviors::PlayerEnergy, (With<super::Player>, Changed<crate::behaviors::PlayerEnergy>)>,
	mut energy_bar_query: Query<&mut Node, With<EnergyBarForeground>>,
	mut energy_text_query: Query<&mut Text, With<EnergyText>>,
) {
	// Only update if UI exists
	let Ok(mut node) = energy_bar_query.single_mut() else {
		return;
	};

	let Ok(mut text) = energy_text_query.single_mut() else {
		return;
	};

	// Only update if energy changed
	if let Ok(energy) = player_query.single() {
		// Update energy bar width
		let energy_percent = (energy.current / energy.max).clamp(0.0, 1.0);
		let new_width = crate::constants::ENERGY_BAR_WIDTH * energy_percent;
		node.width = Val::Px(new_width);

		// Update energy text
		**text = format!("Energy: {:.0}/{:.0}", energy.current, energy.max);
	}
}
