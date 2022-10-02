use crate::{history::TimeSinceLevelStart, AssetHolder, GameState};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::Gameplay, spawn_counter)
            .add_system(update_counter.run_in_state(GameState::Gameplay));
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct Counter;

fn spawn_counter(mut commands: Commands, asset_holder: Res<AssetHolder>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(5.),
                    left: Val::Percent(50.),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "Counter",
                TextStyle {
                    font: asset_holder.font.clone(),
                    font_size: 64.,
                    color: Color::BLACK,
                },
            ),
            ..default()
        })
        .insert(Counter);
}

fn update_counter(
    mut query: Query<&mut Text, With<Counter>>,
    time_since_level_start: Res<TimeSinceLevelStart>,
    asset_holder: Res<AssetHolder>,
) {
    for mut text in query.iter_mut() {
        *text = Text::from_section(
            format!("{:.2}", 10. - time_since_level_start.0),
            TextStyle {
                font: asset_holder.font.clone(),
                font_size: 64.,
                color: Color::BLACK,
            },
        );
    }
}
