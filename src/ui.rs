use crate::{history::TimeSinceLevelStart, player::Vitality, AssetHolder, GameState};
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SpawnWorld, spawn_counter)
            .add_system(update_counter.run_in_state(GameState::Gameplay))
            .add_system(death_screen.run_in_state(GameState::Gameplay));
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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct DeathScreen;

fn death_screen(
    mut commands: Commands,
    vitals: Query<&Vitality, Changed<Vitality>>,
    existing_death_screens: Query<Entity, With<DeathScreen>>,
    asset_holder: Res<AssetHolder>,
) {
    for changed_vitality in vitals.iter() {
        for entity in existing_death_screens.iter() {
            commands.entity(entity).despawn_recursive();
        }

        if *changed_vitality == Vitality::Dead {
            commands
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size {
                            width: Val::Percent(100.),
                            height: Val::Percent(100.),
                        },
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        align_content: AlignContent::Center,
                        ..default()
                    },
                    color: UiColor(Color::rgba(0., 0., 0., 0.8)),
                    ..default()
                })
                .insert(DeathScreen)
                .with_children(|builder| {
                    builder.spawn_bundle(TextBundle {
                        text: Text::from_section(
                            "DEAD\n\nPress R to restart..",
                            TextStyle {
                                font: asset_holder.font.clone(),
                                font_size: 128.,
                                color: Color::WHITE,
                            },
                        ),
                        ..default()
                    });
                });
        }
    }
}
