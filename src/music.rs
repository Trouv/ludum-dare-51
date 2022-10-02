use crate::{
    history::{TimeEvent, TimeScale, TimeSinceLevelStart},
    player::Vitality,
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use iyes_loopless::prelude::*;

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(play_music.run_on_event::<TimeEvent>().after("update_time"))
            .add_system(death_sound.run_in_state(GameState::Gameplay))
            .add_enter_system(GameState::Preamble, victory_sound);
    }
}

fn play_music(
    asset_holder: Res<AssetHolder>,
    audio: Res<bevy_kira_audio::Audio>,
    time_scale: Res<TimeScale>,
    time_since_level_start: Res<TimeSinceLevelStart>,
    vitality: Query<&Vitality>,
) {
    if *vitality.single() == Vitality::Alive {
        audio.stop();
        let mut audio_commands = audio.play(asset_holder.music.clone());

        audio_commands
            .start_from(time_since_level_start.0 as f64 % 10.)
            .with_playback_rate((time_scale.0.abs()).sqrt().sqrt() as f64)
            .looped()
            .with_volume(0.3);

        if time_scale.0 < 0. {
            audio_commands.reverse();
        }
    }
}

fn victory_sound(asset_holder: Res<AssetHolder>, audio: Res<bevy_kira_audio::Audio>) {
    audio.stop();
    audio.play(asset_holder.victory.clone());
}

fn death_sound(
    query: Query<&Vitality, Changed<Vitality>>,
    asset_holder: Res<AssetHolder>,
    audio: Res<bevy_kira_audio::Audio>,
) {
    if query.iter().any(|v| *v == Vitality::Dead) {
        audio.stop();
        audio.play(asset_holder.death.clone());
    }
}
