use crate::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Gameplay, spawn_first_level);
    }
}

fn spawn_first_level(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    query: Query<Entity, With<Handle<LdtkAsset>>>,
) {
    if query.is_empty() {
        commands.spawn_bundle(LdtkWorldBundle {
            ldtk_handle: asset_holder.ldtk.clone(),
            ..default()
        });
    }
}
