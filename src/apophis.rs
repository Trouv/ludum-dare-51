use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    animation::SpriteSheetAnimation, history::TimeSinceLevelStart, AssetHolder, GameState,
};

pub struct ApophisPlugin;

impl Plugin for ApophisPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SpawnWorld, spawn_apophis)
            .add_system(update_apophis_by_time.run_in_state(GameState::Gameplay));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct Apophis;

fn spawn_apophis(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_atlas = atlases.add(TextureAtlas::from_grid(
        asset_holder.apophis.clone(),
        Vec2::splat(128.),
        10,
        1,
    ));
    commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas,
            ..default()
        })
        .insert(SpriteSheetAnimation {
            indices: 0..10,
            frame_timer: Timer::from_seconds(0.1, true),
            repeat: true,
        })
        .insert(Apophis);
}

fn update_apophis_by_time(
    time_since_level_start: Res<TimeSinceLevelStart>,
    mut query: Query<&mut Transform, With<Apophis>>,
) {
    for mut transform in query.iter_mut() {
        let start_translation = Vec3::new(900., 900., 0.);
        let final_translation = Vec3::new(500., 500., 0.);

        let start_scale = Vec3::new(2., 2., 1.);
        let final_scale = Vec3::new(4., 4., 1.);
        let lerp_value = time_since_level_start.0 / 10.;

        *transform =
            Transform::from_translation(start_translation.lerp(final_translation, lerp_value))
                .with_scale(start_scale.lerp(final_scale, lerp_value));
    }
}
