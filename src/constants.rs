use bevy::prelude::*;

// ============ Physics Constants ============

pub const GRAVITY: f32 = -980.0;
pub const GROUND_SNAP_DISTANCE: f32 = 10.0;

// ============ Player Constants ============

pub const PLAYER_DEFAULT_SPEED: f32 = 300.0;
pub const PLAYER_DEFAULT_JUMP_FORCE: f32 = 400.0;
pub const PLAYER_DEFAULT_HEALTH: f32 = 100.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 40.0);
pub const PLAYER_SPAWN_POSITION: Vec3 = Vec3::new(0.0, -200.0, 0.0);
pub const PLAYER_COLOR: Color = Color::srgb(0.2, 0.4, 0.9);
pub const PLAYER_ACCELERATION: f32 = 2000.0;
pub const PLAYER_DECELERATION: f32 = 800.0;

// ============ Input Constants ============

pub const GAMEPAD_DEADZONE: f32 = 0.1;

// ============ Enemy Constants ============

pub const ENEMY_SPAWN_TIMER: f32 = 2.0;
pub const ENEMY_SPAWN_DISTANCE: f32 = 700.0;
pub const ENEMY_SPAWN_Y_MIN: f32 = -200.0;
pub const ENEMY_SPAWN_Y_MAX: f32 = 100.0;

pub const WAVE_DURATION: f32 = 30.0;
pub const WAVE_HEALTH_SCALING: f32 = 0.2;
pub const WAVE_SPAWN_RATE_SCALING: f32 = 0.1;
pub const MIN_SPAWN_DURATION: f32 = 0.5;

pub const HEALTH_BAR_HEIGHT: f32 = 4.0;
pub const HEALTH_BAR_OFFSET_Y: f32 = 8.0;

// ============ Experience Constants ============

pub const INITIAL_XP_TO_NEXT_LEVEL: u32 = 100;
pub const XP_LEVEL_SCALING: f32 = 1.5;
pub const XP_ORB_ATTRACTION_RANGE: f32 = 150.0;
pub const XP_ORB_MOVEMENT_SPEED: f32 = 300.0;
pub const XP_ORB_COLLECTION_RANGE: f32 = 30.0;
pub const XP_ORB_SIZE: Vec2 = Vec2::new(15.0, 15.0);
pub const XP_ORB_COLOR: Color = Color::srgb(0.9, 0.7, 0.2);

// ============ Powerup Constants ============

pub const POWERUP_OPTIONS_COUNT: usize = 3;
pub const POWERUP_OVERLAY_ALPHA: f32 = 0.8;

// ============ UI Constants ============

pub const UI_MARGIN: f32 = 10.0;
pub const UI_FONT_SIZE_LARGE: f32 = 40.0;
pub const UI_FONT_SIZE_MEDIUM: f32 = 24.0;
pub const UI_FONT_SIZE_NORMAL: f32 = 20.0;
pub const UI_FONT_SIZE_SMALL: f32 = 16.0;

pub const XP_BAR_WIDTH: f32 = 300.0;
pub const XP_BAR_HEIGHT: f32 = 20.0;
pub const XP_BAR_TOP: f32 = 40.0;
pub const XP_BAR_COLOR_BG: Color = Color::srgb(0.2, 0.2, 0.2);
pub const XP_BAR_COLOR_FG: Color = Color::srgb(0.2, 0.6, 0.9);

pub const POWERUP_BUTTON_WIDTH: f32 = 400.0;
pub const POWERUP_BUTTON_HEIGHT: f32 = 80.0;
pub const POWERUP_BUTTON_PADDING: f32 = 10.0;
pub const POWERUP_BUTTON_GAP: f32 = 20.0;
pub const POWERUP_TITLE_MARGIN: f32 = 30.0;

pub const POWERUP_COLOR_SELECTED: Color = Color::srgb(0.3, 0.3, 0.5);
pub const POWERUP_COLOR_NORMAL: Color = Color::srgb(0.2, 0.2, 0.3);
pub const POWERUP_COLOR_HOVERED: Color = Color::srgb(0.3, 0.3, 0.4);

// ============ Weapon Constants ============

#[allow(dead_code)]  // Documentation constant for auto_shooter.weapon.ron fire_range
pub const AUTO_SHOOTER_DEFAULT_RANGE: f32 = 400.0;

#[allow(dead_code)]  // Reserved for future weapon range powerups
pub const WEAPON_RANGE_BOOST_AMOUNT: f32 = 100.0;

// Weapon upgrade scaling per level
pub const WEAPON_DAMAGE_INCREASE_PER_LEVEL: f32 = 0.2;  // +20% damage per level
pub const WEAPON_COOLDOWN_DECREASE_PER_LEVEL: f32 = 0.1;  // -10% cooldown per level
pub const WEAPON_MIN_COOLDOWN_MULTIPLIER: f32 = 0.5;  // Minimum 50% cooldown
pub const WEAPON_EFFECT_INCREASE_PER_LEVEL: f32 = 0.15;  // +15% effects (stun, etc) per level

// ============ Melee Attack Constants ============

// Movement speed when tracking enemies during melee attacks
pub const MELEE_TRACKING_SPEED: f32 = 800.0;

// How long the melee attack animation/hitbox lasts
#[allow(dead_code)]  // Configured in weapon data files
pub const MELEE_ATTACK_DURATION: f32 = 0.2;

// How long enemies remain stunned after being hit
#[allow(dead_code)]  // Configured in weapon data files
pub const MELEE_STUN_DURATION: f32 = 0.3;

// Force applied to knock back enemies
#[allow(dead_code)]  // Configured in weapon data files
pub const MELEE_KNOCKBACK_FORCE: f32 = 400.0;

// ============ Platform Constants ============

pub const PLATFORM_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
