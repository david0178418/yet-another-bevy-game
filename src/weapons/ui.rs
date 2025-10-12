use bevy::prelude::*;

#[derive(Component)]
pub struct WeaponCooldownBar {
	pub weapon_entity: Entity,
}

#[derive(Component)]
pub(crate) struct WeaponCooldownBarBackground;

#[derive(Component)]
pub(crate) struct WeaponCooldownBarForeground;

#[derive(Component)]
pub(crate) struct WeaponCooldownText;

#[derive(Component)]
pub struct HasCooldownUI;

pub struct BarLayout {
	pub width: f32,
	pub height: f32,
	pub start_y: f32,
	pub spacing: f32,
}

type NewProjectileWeaponsQuery<'w, 's> = Query<
	'w,
	's,
	(Entity, &'static super::WeaponName),
	(
		With<crate::behaviors::ProjectileSpawner>,
		Without<HasCooldownUI>,
	),
>;
type NewMeleeWeaponsQuery<'w, 's> = Query<
	'w,
	's,
	(Entity, &'static super::WeaponName),
	(With<crate::behaviors::MeleeAttack>, Without<HasCooldownUI>),
>;

pub fn spawn_weapon_cooldown_bars(
	mut commands: Commands,
	projectile_weapons: NewProjectileWeaponsQuery,
	melee_weapons: NewMeleeWeaponsQuery,
	existing_projectile_weapons: Query<
		Entity,
		(
			With<crate::behaviors::ProjectileSpawner>,
			With<HasCooldownUI>,
		),
	>,
	existing_melee_weapons: Query<
		Entity,
		(With<crate::behaviors::MeleeAttack>, With<HasCooldownUI>),
	>,
) {
	const LAYOUT: BarLayout = BarLayout {
		width: 200.0,
		height: 15.0,
		start_y: 10.0,
		spacing: 25.0,
	};

	// Start bar index after existing weapons
	let mut bar_index =
		existing_projectile_weapons.iter().count() + existing_melee_weapons.iter().count();

	// Spawn bars for projectile weapons
	for (entity, weapon_name) in projectile_weapons.iter() {
		spawn_cooldown_bar(&mut commands, entity, &weapon_name.0, bar_index, &LAYOUT);
		bar_index += 1;
	}

	// Spawn bars for melee weapons
	for (entity, weapon_name) in melee_weapons.iter() {
		spawn_cooldown_bar(&mut commands, entity, &weapon_name.0, bar_index, &LAYOUT);
		bar_index += 1;
	}
}

fn spawn_cooldown_bar(
	commands: &mut Commands,
	weapon_entity: Entity,
	weapon_name: &str,
	index: usize,
	layout: &BarLayout,
) {
	let y_position = layout.start_y + (index as f32 * layout.spacing);

	// Mark weapon as having UI
	commands.entity(weapon_entity).insert(HasCooldownUI);

	// Spawn background bar
	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(y_position),
			right: Val::Px(10.0),
			width: Val::Px(layout.width),
			height: Val::Px(layout.height),
			..default()
		},
		BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
		ZIndex(10),
		WeaponCooldownBar { weapon_entity },
		WeaponCooldownBarBackground,
	));

	// Spawn foreground bar (fills up as cooldown progresses)
	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(y_position),
			right: Val::Px(10.0),
			width: Val::Px(0.0),
			height: Val::Px(layout.height),
			..default()
		},
		BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
		ZIndex(11),
		WeaponCooldownBar { weapon_entity },
		WeaponCooldownBarForeground,
	));

	// Spawn text label
	commands.spawn((
		Text::new(weapon_name),
		Node {
			position_type: PositionType::Absolute,
			top: Val::Px(y_position - 2.0),
			right: Val::Px(15.0),
			..default()
		},
		TextColor(Color::WHITE),
		TextFont {
			font_size: 12.0,
			..default()
		},
		ZIndex(12),
		WeaponCooldownBar { weapon_entity },
		WeaponCooldownText,
	));
}

pub fn update_weapon_cooldown_bars(
	projectile_weapons: Query<(Entity, &crate::behaviors::ProjectileSpawner)>,
	melee_weapons: Query<(Entity, &crate::behaviors::MeleeAttack)>,
	mut bars: Query<(&WeaponCooldownBar, &mut Node), With<WeaponCooldownBarForeground>>,
) {
	const BAR_WIDTH: f32 = 200.0;

	for (bar, mut node) in bars.iter_mut() {
		// Check if it's a projectile weapon
		if let Ok((_, spawner)) = projectile_weapons.get(bar.weapon_entity) {
			// Full bar when ready, empty when just fired, fills as it cools down
			let readiness = if spawner.cooldown.is_finished() {
				1.0
			} else {
				spawner.cooldown.fraction()
			};
			node.width = Val::Px(BAR_WIDTH * readiness);
			continue;
		}

		// Check if it's a melee weapon
		if let Ok((_, melee)) = melee_weapons.get(bar.weapon_entity) {
			// Full bar when ready, empty when just fired, fills as it cools down
			let readiness = if melee.cooldown.is_finished() {
				1.0
			} else {
				melee.cooldown.fraction()
			};
			node.width = Val::Px(BAR_WIDTH * readiness);
		}
	}
}
