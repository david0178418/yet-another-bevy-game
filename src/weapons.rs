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
pub struct WeaponRegistry {
	weapons: std::collections::HashMap<String, Handle<WeaponData>>,
}

impl WeaponRegistry {
	pub fn get(&self, id: &str) -> Option<&Handle<WeaponData>> {
		self.weapons.get(id)
	}
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
	let weapon_ids = ["orbiting_blade", "auto_shooter"];

	let weapons = weapon_ids
		.iter()
		.map(|id| {
			let path = format!("weapons/{}.weapon.ron", id);
			(id.to_string(), asset_server.load(path))
		})
		.collect();

	commands.insert_resource(WeaponRegistry { weapons });
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

#[derive(Component, Clone)]
pub struct AutoShooterConfig {
	pub projectile_size: (f32, f32),
	pub projectile_color: (f32, f32, f32),
	pub projectile_lifetime: f32,
}

#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub lifetime: Timer,
}

pub fn spawn_weapon_from_data(
    commands: &mut Commands,
    weapon_data: &WeaponData,
    count: u32,
    blade_query: &mut Query<&mut OrbitingBlade>,
) {
    match &weapon_data.weapon_type {
        WeaponTypeData::OrbitingBlade(blade_data) => {
            spawn_orbiting_blade_from_data(commands, blade_data, count, blade_query);
        }
        WeaponTypeData::AutoShooter(shooter_data) => {
            spawn_auto_shooter_from_data(commands, shooter_data);
        }
    }
}

fn spawn_orbiting_blade_from_data(
    commands: &mut Commands,
    blade_data: &OrbitingBladeData,
    count: u32,
    blade_query: &mut Query<&mut OrbitingBlade>,
) {

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

fn spawn_auto_shooter_from_data(
    commands: &mut Commands,
    shooter_data: &AutoShooterData,
) {
    commands.spawn((
        AutoShooter {
            cooldown: Timer::from_seconds(shooter_data.cooldown, TimerMode::Repeating),
            damage: shooter_data.damage,
            projectile_speed: shooter_data.projectile_speed,
        },
        AutoShooterConfig {
            projectile_size: shooter_data.projectile_size,
            projectile_color: shooter_data.projectile_color,
            projectile_lifetime: shooter_data.projectile_lifetime,
        },
    ));
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
    mut shooter_query: Query<(&mut AutoShooter, &AutoShooterConfig)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    enemy_query: Query<&Transform, With<crate::enemy::Enemy>>,
    time: Res<Time<Virtual>>,
) {

    if let Ok(player_transform) = player_query.get_single() {
        for (mut shooter, config) in shooter_query.iter_mut() {
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
                        color: Color::srgb(config.projectile_color.0, config.projectile_color.1, config.projectile_color.2),
                        custom_size: Some(Vec2::new(config.projectile_size.0, config.projectile_size.1)),
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
                        lifetime: Timer::from_seconds(config.projectile_lifetime, TimerMode::Once),
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
