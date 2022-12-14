mod animation;
mod apophis;
mod camera;
mod event_scheduler;
mod from_component;
mod goals;
mod history;
mod level_transition;
mod music;
mod pause;
mod platform;
mod player;
mod previous_component;
mod ui;
mod wall;

use bevy::{prelude::*, render::texture::ImageSettings};
use bevy_asset_loader::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    AssetLoading,
    SpawnWorld,
    Gameplay,
    Preamble,
    PauseMenu,
}

fn main() {
    App::new()
        .add_loopless_state(GameState::AssetLoading)
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::SpawnWorld)
                .with_collection::<AssetHolder>(),
        )
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(AudioPlugin)
        .add_plugin(LdtkPlugin)
        .insert_resource(LdtkSettings {
            level_background: LevelBackground::Nonexistent,
            int_grid_rendering: IntGridRendering::Colorful,
            ..default()
        })
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //.add_plugin(RapierDebugRenderPlugin::default())
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
        .add_plugin(goals::GoalPlugin)
        .add_plugin(music::MusicPlugin)
        .add_plugin(ui::UiPlugin)
        .add_plugin(pause::PausePlugin)
        .add_plugin(apophis::ApophisPlugin)
        .insert_resource(LevelSelection::Index(0))
        .run();
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, AssetCollection)]
struct AssetHolder {
    #[asset(path = "ludum-dare-51.ldtk")]
    pub ldtk: Handle<LdtkAsset>,
    #[asset(path = "music.ogg")]
    pub music: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "victory.ogg")]
    pub victory: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "death.ogg")]
    pub death: Handle<bevy_kira_audio::prelude::AudioSource>,
    #[asset(path = "Carnevalee Freakshow.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "apophis.png")]
    pub apophis: Handle<Image>,
    #[asset(path = "background.png")]
    pub background: Handle<Image>,
}
