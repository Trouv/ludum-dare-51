use crate::{animation::SpriteSheetAnimation, from_component::*, player::ColliderBundle};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct GoalPlugin;

impl Plugin for GoalPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FromComponentPlugin::<Goal, SpriteSheetAnimation>::new())
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
