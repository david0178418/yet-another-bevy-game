# Data-Driven Architecture

This document explains the data-driven design pattern implemented in this game.

## Core Principle

**Systems operate on components. Data files define which components entities get and their values.**

The behavior stays in code (systems), but content is defined in data files.

## Architecture Overview

```
RON Data Files → Asset Loader → Data Structures → Spawn Functions → Entities with Components
                                                                           ↓
                                                                    Generic Systems
```

## Example: Enemies

### 1. Data File (assets/enemies/weak.enemy.ron)
```ron
(
	name: "Weak Enemy",
	color: (0.8, 0.2, 0.2),
	base_health: 15.0,
	speed: 80.0,
	damage: 10.0,
	size: (30.0, 30.0),
	xp_value: 5,
)
```

### 2. Data Structure (src/enemy.rs)
```rust
#[derive(Asset, TypePath, Deserialize, Clone)]
pub struct EnemyData {
	pub name: String,
	pub color: (f32, f32, f32),
	pub base_health: f32,
	pub speed: f32,
	pub damage: f32,
	pub size: (f32, f32),
	pub xp_value: u32,
}
```

### 3. Asset Loader
```rust
impl AssetLoader for EnemyDataLoader {
	type Asset = EnemyData;
	// Loads .enemy.ron files into EnemyData assets
}
```

### 4. Spawn Function
Reads data and creates entity with appropriate components:
```rust
let enemy_entity = commands.spawn((
	Sprite { color: Color::srgb(enemy_data.color.0, ...), ... },
	Transform::from_xyz(...),
	Enemy {
		health: enemy_data.base_health,
		speed: enemy_data.speed,
		damage: enemy_data.damage,
		xp_value: enemy_data.xp_value,
	},
	Velocity { x: 0.0, y: 0.0 },
	Grounded(false),
));
```

### 5. Generic Systems
Systems don't care about data - they just query components:
```rust
fn move_enemies(
	mut enemy_query: Query<(&Transform, &mut Velocity, &Enemy)>,
	player_query: Query<&Transform, With<Player>>,
) {
	// Behavior defined in code
	// Works on ANY entity with Enemy + Velocity + Transform
}
```

## Example: Weapons (More Complex)

Weapons demonstrate **discriminated unions** - different weapon types need different components.

### Data File (assets/weapons/orbiting_blade.weapon.ron)
```ron
(
	name: "Orbiting Blade",
	weapon_type: OrbitingBlade((
		radius: 80.0,
		speed: 3.0,
		damage: 200.0,
		size: (20.0, 10.0),
		color: (0.8, 0.8, 0.9),
	)),
)
```

### Discriminated Enum
```rust
#[derive(Deserialize, Clone)]
pub enum WeaponTypeData {
	OrbitingBlade(OrbitingBladeData),
	AutoShooter(AutoShooterData),
}
```

### Spawn Logic
Pattern match determines which components to attach:
```rust
match weapon_data.weapon_type {
	WeaponTypeData::OrbitingBlade(ref blade_data) => {
		// Spawn with OrbitingBlade component
		commands.spawn((
			Sprite { ... },
			Transform { ... },
			OrbitingBlade {
				radius: blade_data.radius,
				speed: blade_data.speed,
				damage: blade_data.damage,
			},
		));
	}
	WeaponTypeData::AutoShooter(ref shooter_data) => {
		// Spawn with AutoShooter component
		commands.spawn((
			AutoShooter {
				cooldown: Timer::from_seconds(shooter_data.cooldown, ...),
				damage: shooter_data.damage,
				projectile_speed: shooter_data.projectile_speed,
			},
		));
	}
}
```

## Key Insights

### 1. Systems are Behavior, Data is Content
- `update_orbiting_blade()` defines how orbiting blades work
- RON files define specific blade configurations

### 2. Components are Capabilities
- Entity with `OrbitingBlade` component → processed by blade system
- Entity with `AutoShooter` component → processed by shooter system
- Entity with `Enemy` component → processed by enemy systems

### 3. Composition Over Configuration
Instead of:
```rust
enum WeaponType { Blade, Shooter }
struct Weapon {
	weapon_type: WeaponType,
	// All possible fields for all weapons
}
```

We use:
```rust
// Different entities with different components
Entity(Sprite, Transform, OrbitingBlade)
Entity(AutoShooter)
```

## Benefits Demonstrated

### Before (Hardcoded)
```rust
const BLADE_DAMAGE: f32 = 200.0;
// Must recompile to change damage
```

### After (Data-Driven)
```ron
damage: 200.0,
// Edit file → asset hot-reloads → instant feedback
```

### Extensibility
Adding a new enemy type:
1. Create `elite.enemy.ron`
2. Load it in `EnemyDefinitions`
3. No code changes needed!

## Next Steps

Future enhancements could include:

1. **Progression trees** - Weapon upgrade paths in data
2. **Wave definitions** - Enemy compositions per wave
3. **Level layouts** - Platform configurations
4. **Balance curves** - XP requirements, scaling formulas
5. **Mod support** - Load user-created RON files

## File Structure

```
assets/
├── enemies/
│   ├── weak.enemy.ron
│   ├── medium.enemy.ron
│   └── strong.enemy.ron
└── weapons/
    ├── orbiting_blade.weapon.ron
    └── auto_shooter.weapon.ron
```

## Hot Reloading

With `file_watcher` feature enabled (already added to Cargo.toml), edit any `.ron` file and see changes instantly in-game without recompiling!

## Summary

**The Pattern:**
1. Define data schema (Rust structs with `Deserialize`)
2. Create asset loader for file format
3. Write RON files with content
4. Spawn function reads data → creates entities with components
5. Generic systems process components (unchanged)

This separates "what things are" (data) from "how things work" (systems).
