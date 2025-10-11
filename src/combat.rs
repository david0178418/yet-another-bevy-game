use bevy::prelude::*;

pub struct CombatPlugin;

type DamageableQuery<'w, 's> = Query<'w, 's, (
    &'static Transform,
    &'static Sprite,
    &'static mut crate::behaviors::Damageable,
    Has<crate::behaviors::EnemyTag>,
    Has<crate::behaviors::PlayerTag>,
)>;

type DeathQuery<'w, 's> = Query<'w, 's, (
    Entity,
    &'static Transform,
    &'static crate::behaviors::Damageable,
    Has<crate::behaviors::EnemyTag>,
    Option<&'static crate::enemy::Enemy>,
)>;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            apply_contact_damage,
            handle_damageable_death,
        )
            .after(crate::physics::PhysicsSet)
            .before(crate::physics::CollisionResolutionSet));
    }
}

// Generic damage-on-contact system
fn apply_contact_damage(
    mut commands: Commands,
    damage_dealers: Query<(Entity, &Transform, &Sprite, &crate::behaviors::DamageOnContact)>,
    mut damageables: DamageableQuery,
    time: Res<Time<Virtual>>,
) {
    use crate::behaviors::*;

    for (dealer_entity, dealer_transform, dealer_sprite, damage_on_contact) in damage_dealers.iter() {
        let dealer_size = dealer_sprite.custom_size.unwrap_or(Vec2::ONE);

        for (target_transform, target_sprite, mut damageable, is_enemy, is_player) in damageables.iter_mut() {
            // Check if target matches the damage filter
            let target_matches = match damage_on_contact.targets {
                TargetFilter::Enemies => is_enemy,
                TargetFilter::Player => is_player,
                TargetFilter::All => true,
            };

            if !target_matches {
                continue;
            }

            let target_size = target_sprite.custom_size.unwrap_or(Vec2::ONE);

            if check_collision(
                dealer_transform.translation,
                dealer_size,
                target_transform.translation,
                target_size,
            ) {
                match damage_on_contact.damage_type {
                    DamageType::Continuous => {
                        damageable.health -= damage_on_contact.damage * time.delta_secs();
                    }
                    DamageType::OneTime => {
                        damageable.health -= damage_on_contact.damage;
                        // Despawn one-time damage dealers (like projectiles)
                        commands.entity(dealer_entity).despawn();
                        break; // Stop after first hit
                    }
                }
            }
        }
    }
}

// Generic death handling
fn handle_damageable_death(
    mut commands: Commands,
    query: DeathQuery,
    health_bar_query: Query<(Entity, &crate::enemy::HealthBar)>,
) {
    for (entity, transform, damageable, is_enemy, enemy_data) in query.iter() {
        if damageable.health <= 0.0 {
            // If it's an enemy, spawn XP orb
            if is_enemy {
                if let Some(enemy) = enemy_data {
                    commands.spawn((
                        Sprite {
                            color: crate::constants::XP_ORB_COLOR,
                            custom_size: Some(crate::constants::XP_ORB_SIZE),
                            ..default()
                        },
                        Transform::from_translation(transform.translation),
                        crate::experience::ExperienceOrb {
                            value: enemy.xp_value,
                        },
                    ));

                    // Despawn health bars
                    for (bar_entity, health_bar) in health_bar_query.iter() {
                        if health_bar.enemy_entity == entity {
                            commands.entity(bar_entity).despawn();
                        }
                    }
                }
            }

            commands.entity(entity).despawn();
        }
    }
}

fn check_collision(pos1: Vec3, size1: Vec2, pos2: Vec3, size2: Vec2) -> bool {
    let half_size1 = size1 / 2.0;
    let half_size2 = size2 / 2.0;

    pos1.x - half_size1.x < pos2.x + half_size2.x
        && pos1.x + half_size1.x > pos2.x - half_size2.x
        && pos1.y - half_size1.y < pos2.y + half_size2.y
        && pos1.y + half_size1.y > pos2.y - half_size2.y
}
