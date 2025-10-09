use bevy::prelude::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            blade_damage_enemies,
            projectile_damage_enemies,
            enemy_damage_player,
        ));
    }
}

fn blade_damage_enemies(
    mut enemy_query: Query<(&mut crate::enemy::Enemy, &Transform, &Sprite)>,
    blade_query: Query<(&Transform, &Sprite, &crate::weapons::OrbitingBlade)>,
    time: Res<Time<Virtual>>,
) {
    for (blade_transform, blade_sprite, blade) in blade_query.iter() {
        let blade_size = blade_sprite.custom_size.unwrap_or(Vec2::ONE);

        for (mut enemy, enemy_transform, enemy_sprite) in enemy_query.iter_mut() {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);

            if check_collision(
                blade_transform.translation,
                blade_size,
                enemy_transform.translation,
                enemy_size,
            ) {
                enemy.health -= blade.damage * time.delta_secs();
            }
        }
    }
}

fn projectile_damage_enemies(
    mut commands: Commands,
    mut enemy_query: Query<(&mut crate::enemy::Enemy, &Transform, &Sprite)>,
    projectile_query: Query<(Entity, &Transform, &Sprite, &crate::weapons::Projectile)>,
) {
    for (projectile_entity, projectile_transform, projectile_sprite, projectile) in projectile_query.iter() {
        let projectile_size = projectile_sprite.custom_size.unwrap_or(Vec2::ONE);

        for (mut enemy, enemy_transform, enemy_sprite) in enemy_query.iter_mut() {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);

            if check_collision(
                projectile_transform.translation,
                projectile_size,
                enemy_transform.translation,
                enemy_size,
            ) {
                enemy.health -= projectile.damage;
                commands.entity(projectile_entity).despawn();
                break; // Projectile can only hit one enemy
            }
        }
    }
}

fn enemy_damage_player(
    mut player_query: Query<(&mut crate::player::Player, &Transform, &Sprite)>,
    enemy_query: Query<(&crate::enemy::Enemy, &Transform, &Sprite)>,
    time: Res<Time<Virtual>>,
) {
    if let Ok((mut player, player_transform, player_sprite)) = player_query.single_mut() {
        let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);

        for (enemy, enemy_transform, enemy_sprite) in enemy_query.iter() {
            let enemy_size = enemy_sprite.custom_size.unwrap_or(Vec2::ONE);

            if check_collision(
                player_transform.translation,
                player_size,
                enemy_transform.translation,
                enemy_size,
            ) {
                player.health -= enemy.damage * time.delta_secs();
                player.health = player.health.max(0.0);
            }
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
