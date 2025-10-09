use bevy::{prelude::*, asset::AssetLoader};
use std::f32::consts::PI;
use serde::Deserialize;

pub struct WeaponsPlugin;

// Data-driven weapon definitions
#[derive(Deserialize, Clone)]
pub struct OrbitingBladeData {
	pub radius: f32,
	pub speed: f32,
	pub damage: f32,
	pub size: (f32, f32),
	pub color: (f32, f32, f32),
}

#[derive(Deserialize, Clone)]
pub struct AutoShooterData {
	pub cooldown: f32,
	pub damage: f32,
	pub projectile_speed: f32,
	pub projectile_size: (f32, f32),
	pub projectile_color: (f32, f32, f32),
	pub projectile_lifetime: f32,
}

#[derive(Deserialize, Clone)]
pub enum WeaponTypeData {
	OrbitingBlade(OrbitingBladeData),
	AutoShooter(AutoShooterData),
}

#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct WeaponData {
	pub weapon_type: WeaponTypeData,
}

#[derive(Default)]
struct WeaponDataLoader;

impl AssetLoader for WeaponDataLoader {
	type Asset = WeaponData;
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
		let data = ron::de::from_bytes::<WeaponData>(&bytes)
			.map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
		Ok(data)
	}

	fn extensions(&self) -> &[&str] {
		&["weapon.ron"]
	}
}

#[derive(Resource)]
pub struct WeaponDefinitions {
	pub orbiting_blade: Handle<WeaponData>,
	pub auto_shooter: Handle<WeaponData>,
}

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<WeaponData>()
            .init_asset_loader::<WeaponDataLoader>()
            .add_systems(Startup, load_weapon_definitions)
            .add_systems(Update, (
                update_orbiting_blade,
                update_auto_shooter,
                move_projectiles,
            ));
    }
}

fn load_weapon_definitions(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(WeaponDefinitions {
		orbiting_blade: asset_server.load("weapons/orbiting_blade.weapon.ron"),
		auto_shooter: asset_server.load("weapons/auto_shooter.weapon.ron"),
	});
}

#[derive(Component)]
pub struct OrbitingBlade {
    pub angle: f32,
    pub radius: f32,
    pub speed: f32,
    pub damage: f32,
}

#[derive(Component)]
pub struct AutoShooter {
    pub cooldown: Timer,
    pub damage: f32,
    pub projectile_speed: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: Timer,
}

pub fn spawn_orbiting_blade(
    commands: &mut Commands,
    count: u32,
    blade_query: &mut Query<&mut OrbitingBlade>,
    weapon_defs: Option<&WeaponDefinitions>,
    weapon_data_assets: &Assets<WeaponData>,
) {
    let Some(defs) = weapon_defs else { return };
    let Some(weapon_data) = weapon_data_assets.get(&defs.orbiting_blade) else { return };
    let WeaponTypeData::OrbitingBlade(ref blade_data) = weapon_data.weapon_type else { return };

    let existing_count = blade_query.iter().count();
    let total_count = existing_count + count as usize;

    // Redistribute existing blades
    for (index, mut blade) in blade_query.iter_mut().enumerate() {
        blade.angle = (index as f32 / total_count as f32) * 2.0 * PI;
    }

    // Spawn new blades with evenly distributed angles
    for i in 0..count {
        let blade_index = existing_count + i as usize;
        commands.spawn((
            Sprite {
                color: Color::srgb(blade_data.color.0, blade_data.color.1, blade_data.color.2),
                custom_size: Some(Vec2::new(blade_data.size.0, blade_data.size.1)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 1.0),
            OrbitingBlade {
                angle: (blade_index as f32 / total_count as f32) * 2.0 * PI,
                radius: blade_data.radius,
                speed: blade_data.speed,
                damage: blade_data.damage,
            },
        ));
    }
}

pub fn spawn_auto_shooter(
    commands: &mut Commands,
    weapon_defs: Option<&WeaponDefinitions>,
    weapon_data_assets: &Assets<WeaponData>,
) {
    let Some(defs) = weapon_defs else { return };
    let Some(weapon_data) = weapon_data_assets.get(&defs.auto_shooter) else { return };
    let WeaponTypeData::AutoShooter(ref shooter_data) = weapon_data.weapon_type else { return };

    commands.spawn(AutoShooter {
        cooldown: Timer::from_seconds(shooter_data.cooldown, TimerMode::Repeating),
        damage: shooter_data.damage,
        projectile_speed: shooter_data.projectile_speed,
    });
}

fn update_orbiting_blade(
    mut blade_query: Query<(&mut Transform, &mut OrbitingBlade)>,
    player_query: Query<&Transform, (With<crate::player::Player>, Without<OrbitingBlade>)>,
    time: Res<Time<Virtual>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (mut blade_transform, mut blade) in blade_query.iter_mut() {
            blade.angle += blade.speed * time.delta_secs();

            blade_transform.translation.x = player_transform.translation.x + blade.angle.cos() * blade.radius;
            blade_transform.translation.y = player_transform.translation.y + blade.angle.sin() * blade.radius;
            blade_transform.rotation = Quat::from_rotation_z(blade.angle + PI / 2.0);
        }
    }
}

fn update_auto_shooter(
    mut commands: Commands,
    mut shooter_query: Query<&mut AutoShooter>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    enemy_query: Query<&Transform, With<crate::enemy::Enemy>>,
    time: Res<Time<Virtual>>,
    weapon_defs: Option<Res<WeaponDefinitions>>,
    weapon_data_assets: Res<Assets<WeaponData>>,
) {
    let Some(defs) = weapon_defs else { return };
    let Some(weapon_data) = weapon_data_assets.get(&defs.auto_shooter) else { return };
    let WeaponTypeData::AutoShooter(ref shooter_data) = weapon_data.weapon_type else { return };

    if let Ok(player_transform) = player_query.get_single() {
        for mut shooter in shooter_query.iter_mut() {
            if shooter.cooldown.tick(time.delta()).just_finished() {
                // Find nearest enemy
                let mut nearest_distance = f32::MAX;
                let mut nearest_direction = 1.0;

                for enemy_transform in enemy_query.iter() {
                    let distance = player_transform.translation.distance(enemy_transform.translation);
                    if distance < nearest_distance {
                        nearest_distance = distance;
                        nearest_direction = (enemy_transform.translation.x - player_transform.translation.x).signum();
                    }
                }

                // Spawn projectile
                commands.spawn((
                    Sprite {
                        color: Color::srgb(shooter_data.projectile_color.0, shooter_data.projectile_color.1, shooter_data.projectile_color.2),
                        custom_size: Some(Vec2::new(shooter_data.projectile_size.0, shooter_data.projectile_size.1)),
                        ..default()
                    },
                    Transform::from_xyz(
                        player_transform.translation.x + nearest_direction * 30.0,
                        player_transform.translation.y,
                        0.0,
                    ),
                    crate::physics::Velocity {
                        x: nearest_direction * shooter.projectile_speed,
                        y: 0.0,
                    },
                    Projectile {
                        damage: shooter.damage,
                        lifetime: Timer::from_seconds(shooter_data.projectile_lifetime, TimerMode::Once),
                    },
                ));
            }
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Projectile)>,
    time: Res<Time<Virtual>>,
) {
    for (entity, mut projectile) in projectile_query.iter_mut() {
        if projectile.lifetime.tick(time.delta()).just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
