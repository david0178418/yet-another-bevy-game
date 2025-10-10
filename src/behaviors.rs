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

// ============ Spawning Behaviors ============

#[derive(Component)]
pub struct ProjectileSpawner {
	pub cooldown: Timer,
	pub projectile_template: ProjectileTemplate,
	pub spawn_logic: SpawnLogic,
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
	},
	FollowPlayer,
}

// ============ Utility Component ============

#[derive(Component)]
pub struct DespawnOnTimer {
	pub timer: Timer,
}
