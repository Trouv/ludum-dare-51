use crate::{
    history::{History, Moment, TimeEvent, TimeScale, TimeSinceLevelStart},
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::ldtk_grid_coords_to_translation_centered};
use bevy_kira_audio::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(play_music.run_on_event::<TimeEvent>().after("update_time"));
    }
}

fn play_music(
    asset_holder: Res<AssetHolder>,
    audio: Res<bevy_kira_audio::Audio>,
    time_scale: Res<TimeScale>,
    time_since_level_start: Res<TimeSinceLevelStart>,
) {
    audio.stop();
    let mut audio_commands = audio.play(asset_holder.music.clone());

    audio_commands
        .start_from(time_since_level_start.0 as f64 % 10.)
        .with_playback_rate((time_scale.0.abs()).sqrt().sqrt().sqrt() as f64)
        .looped()
        .with_volume(0.5);

    if time_scale.0 < 0. {
        audio_commands.reverse();
    }
}
