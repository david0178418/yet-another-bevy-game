use bevy::prelude::*;

// ============ Physics Constants ============

pub const GRAVITY: f32 = -980.0;
pub const GROUND_SNAP_DISTANCE: f32 = 10.0;

// ============ Player Constants ============

pub const PLAYER_DEFAULT_SPEED: f32 = 200.0;
pub const PLAYER_DEFAULT_JUMP_FORCE: f32 = 400.0;
pub const PLAYER_DEFAULT_HEALTH: f32 = 100.0;
pub const PLAYER_SIZE: Vec2 = Vec2::new(40.0, 40.0);
pub const PLAYER_SPAWN_POSITION: Vec3 = Vec3::new(0.0, -200.0, 0.0);
pub const PLAYER_COLOR: Color = Color::srgb(0.2, 0.4, 0.9);
pub const PLAYER_ACCELERATION: f32 = 1000.0;
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

pub const PLATFORM_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
