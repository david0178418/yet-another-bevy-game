# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## High-Level Architecture

This is a Vampire Survivors-style platformer game built with Bevy 0.17. The game features auto-attacking weapons, enemy waves, experience/leveling, and a powerup system.

### Plugin-Based Architecture

The game is organized into independent plugins that communicate through Bevy's ECS:

- `PhysicsPlugin` - Custom platformer physics with gravity, velocity, ground detection, and entity collision
- `PlayerPlugin` - Player spawning, movement (keyboard/gamepad), jumping, UI
- `EnemyPlugin` - Enemy spawning, wave system, health bars
- `WeaponsPlugin` - Weapon system with data-driven behaviors
- `CombatPlugin` - Damage application, death handling, explosion logic
- `MovementPlugin` - AI movement behaviors (seek, zigzag, maintain distance)
- `ExperiencePlugin` - XP orbs, leveling system
- `PowerupsPlugin` - Level-up menu with weapon and stat upgrades

### Data-Driven Design with RON Files

Weapons and enemies are configured in `.ron` files under `assets/`:

- **Weapons** (`assets/weapons/*.weapon.ron`): Define visual appearance, behaviors (Orbiting, ProjectileSpawner, MeleeAttack), and upgrade behaviors (ScaleDamage, ReduceCooldown, SpawnAdditionalEntity)
- **Enemies** (`assets/enemies/*.enemy.ron`): Define health, size, XP value, and behaviors (SeekTarget, ZigZagMovement, MaintainDistance, ProjectileSpawner, ExplodeOnProximity)
- **Game Config** (`assets/game_config.ron`): Lists all available weapons/enemies and defines the powerup pool

The weapon and enemy registries are populated from `game_config.ron` at startup, loading the specified assets dynamically.

### Behavior Component System

The `behaviors.rs` module defines reusable components that can be attached to any entity:

**Movement Behaviors:**
- `OrbitingBehavior` - Circle around a point
- `FollowPlayer` - Track player position
- `SeekTarget` - Move toward player or nearest enemy
- `ZigZagMovement` - Oscillating path toward target
- `MaintainDistance` - Keep specific distance from target

**Combat Behaviors:**
- `DamageOnContact` - Apply damage when touching (Continuous or OneTime)
- `ProjectileSpawner` - Spawn projectiles with various spawn logic patterns
- `MeleeAttack` - Close-range attack with hitbox, stun, and knockback
- `ExplodeOnProximity` - Trigger explosion when near target

**Weapon Stats and Upgrades:**
- Weapons compose `DamageStats`, `CooldownStats`, and `EffectStats` components
- `UpgradeBehaviors` component defines how weapons scale when leveled up
- The `upgrades.rs` system reads weapon levels and applies stat modifications accordingly

### Physics System

Custom platformer physics in `physics.rs`:

- Entities have `Velocity`, `Grounded`, and `Collider` components
- `PhysicsSet` runs gravity → velocity → ground collision in sequence
- `CollisionResolutionSet` runs after physics to prevent entity overlap
- Ground detection uses axis-aligned bounding box (AABB) collision with snap distance
- Entity-to-entity collision resolution pushes entities apart on the axis with least overlap

### Fixed Viewport System

The game renders at a fixed 1280x720 resolution regardless of window size:

- `calculate_viewport()` creates letterboxing/pillarboxing to maintain 16:9 aspect ratio
- `calculate_ui_scale()` scales UI elements proportionally
- `update_camera_viewport()` handles window resize events

## Workflow Notes

For all new implementations, the code should have no awareness of entities such as specific weapon or enemy types.  New additions should be decomposed into generalized systems and components and defined in assets as an ron file.

When adding a new weapon:
1. Create a `.weapon.ron` file in `assets/weapons/`
2. Add the weapon ID to `weapon_ids` in `assets/game_config.ron`
3. Optionally add it to the `powerup_pool`

When adding a new enemy:
1. Create a `.enemy.ron` file in `assets/enemies/`
2. Add the enemy ID to `enemy_ids` in `assets/game_config.ron`
3. Enemies spawn randomly from the configured pool

When adding a new behavior:
1. Define the component struct in `behaviors.rs`
2. Add a variant to `BehaviorData` enum for RON deserialization
3. Implement the behavior system (movement in `movement.rs`, combat in `combat.rs`, weapon-specific in `weapons/behaviors.rs`)
4. Add behavior application logic in the appropriate spawn function (`enemy.rs::apply_enemy_behaviors` or `weapons/mod.rs::spawn_entity_from_data`)
