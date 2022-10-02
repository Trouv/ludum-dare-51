use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::{
    animation::SpriteSheetAnimation, history::TimeSinceLevelStart, AssetHolder, GameState,
};

pub struct ApophisPlugin;

impl Plugin for ApophisPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SpawnWorld, spawn_apophis)
            .add_system(
                update_apophis_by_time
                    .run_in_state(GameState::Gameplay)
                    .before("parallax"),
            )
            .add_system(
                update_background
                    .run_in_state(GameState::Gameplay)
                    .before("parallax"),
            )
            .add_system(parallax.run_in_state(GameState::Gameplay).label("parallax"));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct Apophis;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct Background;

#[derive(Copy, Clone, PartialEq, Debug, Default, Component)]
struct ParallaxScale(f32);

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
        .insert(ParallaxScale(0.6))
        .insert(SpriteSheetAnimation {
            indices: 0..10,
            frame_timer: Timer::from_seconds(0.1, true),
            repeat: true,
        })
        .insert(Apophis);

    // spawn background
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgba(1., 1., 1., 0.7),
                ..default()
            },
            texture: asset_holder.background.clone(),
            //transform: Transform::from_xyz(960., 540., 0.).with_scale(Vec3::new(1.5, 1.5, 1.)),
            ..default()
        })
        .insert(Background)
        .insert(ParallaxScale(0.8));
}

fn update_apophis_by_time(
    time_since_level_start: Res<TimeSinceLevelStart>,
    mut query: Query<&mut Transform, With<Apophis>>,
) {
    for mut transform in query.iter_mut() {
        let start_translation = Vec3::new(1200., 800., 1.);
        let final_translation = Vec3::new(800., 400., 1.);

        let start_scale = Vec3::new(2., 2., 1.);
        let final_scale = Vec3::new(4., 4., 1.);
        let lerp_value = time_since_level_start.0 / 10.;

        *transform =
            Transform::from_translation(start_translation.lerp(final_translation, lerp_value))
                .with_scale(start_scale.lerp(final_scale, lerp_value));
    }
}

fn update_background(mut query: Query<&mut Transform, With<Background>>) {
    for mut transform in query.iter_mut() {
        *transform = Transform::from_xyz(960., 540., 0.).with_scale(Vec3::new(1.5, 1.5, 1.));
    }
}

fn parallax(
    camera_query: Query<&Transform, With<Camera>>,
    mut parallax_query: Query<(&mut Transform, &ParallaxScale), Without<Camera>>,
) {
    for (mut transform, parallax) in parallax_query.iter_mut() {
        let camera_transform = camera_query.single();

        transform.translation += (camera_transform.translation.truncate() * parallax.0).extend(0.);
    }
}
