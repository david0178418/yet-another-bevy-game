use bevy::prelude::*;
use serde::Deserialize;

// ============ Movement Behaviors ============

#[derive(Component)]
pub struct OrbitingBehavior {
	pub radius: f32,
	pub speed: f32,
	pub angle: f32,
}

#[derive(Component)]
#[allow(dead_code)]
pub struct FollowEntity {
	pub target_entity: Entity,
}

#[derive(Component)]
pub struct FollowPlayer;

// ============ Damage Behaviors ============

#[derive(Component)]
pub struct DamageOnContact {
	pub damage: f32,
	pub damage_type: DamageType,
	pub targets: TargetFilter,
}

#[derive(Clone, Copy, Deserialize)]
pub enum DamageType {
	Continuous,
	OneTime,
}

#[derive(Clone, Copy, Deserialize)]
pub enum TargetFilter {
	Enemies,
	Player,
	All,
}

#[derive(Component)]
pub struct Damageable {
	pub health: f32,
	pub max_health: f32,
}

// ============ Target Tags ============

#[derive(Component)]
pub struct PlayerTag;

#[derive(Component)]
pub struct EnemyTag;

#[derive(Component)]
pub struct ProjectileTag;

// ============ Range Detection ============

#[derive(Component)]
#[allow(dead_code)]  // Reserved for future use (XP attraction, pickups, area triggers)
pub struct ProximityDetector {
	pub range: f32,
	pub target_filter: TargetFilter,
}

// ============ Spawning Behaviors ============

#[derive(Component)]
pub struct ProjectileSpawner {
	pub cooldown: Timer,
	pub projectile_template: ProjectileTemplate,
	pub spawn_logic: SpawnLogic,
	pub fire_range: Option<f32>,  // None = infinite range
}

#[derive(Clone)]
pub struct ProjectileTemplate {
	pub damage: f32,
	pub speed: f32,
	pub lifetime: f32,
	pub size: (f32, f32),
	pub color: (f32, f32, f32),
}

#[derive(Clone, Deserialize)]
pub enum SpawnLogic {
	NearestEnemy,
	PlayerDirection,
	Fixed(f32, f32),
}

// ============ Melee Behaviors ============

#[derive(Component)]
pub struct MeleeAttack {
	pub cooldown: Timer,
	pub detection_range: f32,
	pub shock_wave_damage: f32,
	pub shock_wave_size: (f32, f32),
	pub shock_wave_speed: f32,
	pub shock_wave_travel_distance: f32,
	pub shock_wave_color: (f32, f32, f32),
}

#[derive(Component)]
pub struct DashState {
	pub distance_traveled: f32,
	pub direction: Vec2,
	pub shock_wave_params: ShockWaveParams,
}

#[derive(Clone)]
pub struct ShockWaveParams {
	pub damage: f32,
	pub size: (f32, f32),
	pub speed: f32,
	pub travel_distance: f32,
	pub color: (f32, f32, f32),
}

#[derive(Component)]
pub struct ShockWave {
	pub distance_traveled: f32,
	pub max_distance: f32,
}

// ============ Data Structures for Deserialization ============

#[derive(Deserialize, Clone)]
pub enum BehaviorData {
	Orbiting {
		radius: f32,
		speed: f32,
	},
	DamageOnContact {
		damage: f32,
		damage_type: DamageType,
		targets: TargetFilter,
	},
	ProjectileSpawner {
		cooldown: f32,
		damage: f32,
		speed: f32,
		lifetime: f32,
		projectile_size: (f32, f32),
		projectile_color: (f32, f32, f32),
		spawn_logic: SpawnLogic,
		fire_range: Option<f32>,
	},
	MeleeAttack {
		cooldown: f32,
		detection_range: f32,
		#[allow(dead_code)]  // Used in data files but constants used at runtime
		dash_speed: f32,
		#[allow(dead_code)]  // Used in data files but constants used at runtime
		dash_distance: f32,
		shock_wave_damage: f32,
		shock_wave_size: (f32, f32),
		shock_wave_speed: f32,
		shock_wave_travel_distance: f32,
		shock_wave_color: (f32, f32, f32),
	},
	FollowPlayer,
}

// ============ Utility Component ============

#[derive(Component)]
pub struct DespawnOnTimer {
	pub timer: Timer,
}
