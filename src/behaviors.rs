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
	pub damage: f32,
	pub stun_duration: f32,
	pub knockback_force: f32,
	pub attack_duration: f32,
	pub hitbox_size: (f32, f32),
	pub hitbox_color: (f32, f32, f32),
}

#[derive(Component)]
pub struct MeleeAttackState {
	pub attack_timer: Timer,
	pub damage: f32,
	pub stun_duration: f32,
	pub knockback_force: f32,
	pub hitbox_size: (f32, f32),
	pub hitbox_color: (f32, f32, f32),
	pub attack_direction: Vec2,
}

#[derive(Component)]
pub struct MeleeHitbox {
	pub damage: f32,
	pub stun_duration: f32,
	pub knockback_force: f32,
	pub hit_entities: Vec<Entity>,
}

#[derive(Component)]
pub struct Stunned {
	pub timer: Timer,
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
		damage: f32,
		stun_duration: f32,
		knockback_force: f32,
		attack_duration: f32,
		hitbox_size: (f32, f32),
		hitbox_color: (f32, f32, f32),
	},
	FollowPlayer,
}

// ============ Utility Component ============

#[derive(Component)]
pub struct DespawnOnTimer {
	pub timer: Timer,
}

// ============ Weapon Tracking Components ============

#[derive(Component)]
#[allow(dead_code)]  // Used for weapon tracking, not accessed directly
pub struct WeaponId(pub String);

#[derive(Component, Clone, Copy)]
pub struct WeaponLevel(pub u32);

// ============ Weapon Stat Components ============
// Each weapon composes only the stats it needs

#[derive(Component, Clone, Copy)]
pub struct DamageStats {
	pub base: f32,
}

#[derive(Component, Clone, Copy)]
pub struct CooldownStats {
	pub base: f32,
}

#[derive(Component, Clone, Copy)]
pub struct EffectStats {
	pub base: f32,  // For melee: stun duration, for projectiles: speed, etc.
}

// ============ Upgrade Behavior System ============

#[derive(Clone, Copy, Deserialize)]
pub enum UpgradeBehavior {
	ScaleDamage { per_level: f32 },
	ReduceCooldown { per_level: f32, min_multiplier: f32 },
	IncreaseEffect { per_level: f32 },
	SpawnAdditionalEntity,
}

#[derive(Component, Clone, Deserialize)]
pub struct UpgradeBehaviors(pub Vec<UpgradeBehavior>);
