mod animation;
mod camera;
mod event_scheduler;
mod from_component;
mod history;
mod level_transition;
mod platform;
mod player;
mod wall;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    MainMenu,
    Gameplay,
    PauseMenu,
}

fn main() {
    App::new()
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Gameplay)
                .with_collection::<AssetHolder>(),
        )
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            int_grid_rendering: IntGridRendering::Colorful,
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(RapierDebugRenderPlugin::default())
        .insert_resource(RapierConfiguration {
            gravity: Vec2::ZERO,
            ..default()
        })
        .add_plugin(wall::WallPlugin)
        .add_plugin(player::PlayerPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(level_transition::LevelTransitionPlugin)
        .add_plugin(history::HistoryPlugin)
        .add_plugin(platform::PlatformPlugin)
        .add_plugin(animation::SpriteSheetAnimationPlugin)
        .insert_resource(LevelSelection::Index(0))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, AssetCollection)]
struct AssetHolder {
    #[asset(path = "ludum-dare-51.ldtk")]
    pub ldtk: Handle<LdtkAsset>,
}
