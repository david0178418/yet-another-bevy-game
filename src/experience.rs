use bevy::prelude::*;

pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerExperience {
            current_xp: 0,
            xp_to_next_level: 100,
        })
        .add_event::<LevelUpEvent>()
        .add_systems(Update, (
            move_xp_orbs_to_player,
            collect_experience,
            check_level_up,
        ));
    }
}

#[derive(Resource)]
pub struct PlayerExperience {
    pub current_xp: u32,
    pub xp_to_next_level: u32,
}

#[derive(Event)]
pub struct LevelUpEvent;

#[derive(Component)]
pub struct ExperienceOrb {
    pub value: u32,
}

fn move_xp_orbs_to_player(
    mut orb_query: Query<&mut Transform, (With<ExperienceOrb>, Without<crate::player::Player>)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
    time: Res<Time<Virtual>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for mut orb_transform in orb_query.iter_mut() {
            let distance = player_transform.translation.distance(orb_transform.translation);

            // Attract orbs within range
            if distance < 150.0 {
                let direction = (player_transform.translation - orb_transform.translation).normalize();
                orb_transform.translation += direction * 300.0 * time.delta_secs();
            }
        }
    }
}

fn collect_experience(
    mut commands: Commands,
    mut player_xp: ResMut<PlayerExperience>,
    orb_query: Query<(Entity, &Transform, &ExperienceOrb)>,
    player_query: Query<&Transform, With<crate::player::Player>>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        for (entity, orb_transform, orb) in orb_query.iter() {
            let distance = player_transform.translation.distance(orb_transform.translation);

            if distance < 30.0 {
                player_xp.current_xp += orb.value;
                commands.entity(entity).despawn();
            }
        }
    }
}

fn check_level_up(
    mut player_xp: ResMut<PlayerExperience>,
    mut player_query: Query<&mut crate::player::Player>,
    mut level_up_events: EventWriter<LevelUpEvent>,
) {
    if player_xp.current_xp >= player_xp.xp_to_next_level {
        player_xp.current_xp -= player_xp.xp_to_next_level;
        player_xp.xp_to_next_level = (player_xp.xp_to_next_level as f32 * 1.5) as u32;

        if let Ok(mut player) = player_query.get_single_mut() {
            player.level += 1;
            level_up_events.send(LevelUpEvent);
        }
    }
}
