use crate::{animation::SpriteSheetAnimation, from_component::*, player::*, GameState};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FromComponentPlugin::<Goal, SpriteSheetAnimation>::new())
            .add_system(victory.run_in_state(GameState::Gameplay))
            .register_ldtk_entity::<GoalBundle>("Goal");
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct Goal;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct GoalBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub goal: Goal,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub sensor: Sensor,
}

impl From<Goal> for SpriteSheetAnimation {
    fn from(_: Goal) -> Self {
        SpriteSheetAnimation {
            indices: 72..78,
            repeat: true,
            frame_timer: Timer::from_seconds(0.2, true),
        }
    }
}

fn victory(
    mut commands: Commands,
    player_query: Query<Entity, With<Player>>,
    goal_query: Query<Entity, With<Goal>>,
    mut collision_events: EventReader<CollisionEvent>,
    mut level_selection: ResMut<LevelSelection>,
) {
    for collision in collision_events.iter() {
        match collision {
            CollisionEvent::Started(a, b, _) => {
                if player_query.contains(*a) && goal_query.contains(*b)
                    || player_query.contains(*b) && goal_query.contains(*a)
                {
                    if let LevelSelection::Index(level_index) = *level_selection {
                        *level_selection = LevelSelection::Index(level_index + 1);
                        commands.insert_resource(NextState(GameState::Preamble));
                    }
                }
            }
            _ => (),
        }
    }
}
