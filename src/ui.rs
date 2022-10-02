use crate::{
    history::TimeSinceLevelStart,
    player::Vitality,
    previous_component::{PreviousComponent, PreviousComponentPlugin, TrackPreviousComponent},
    AssetHolder, GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use iyes_loopless::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SpawnWorld, spawn_counter)
            .add_plugin(PreviousComponentPlugin::<Interaction>::default())
            .add_event::<UiAction>()
            .add_system(update_counter.run_in_state(GameState::Gameplay))
            .add_enter_system(GameState::SpawnWorld, spawn_level_num)
            .add_system(update_level_num.run_in_state(GameState::Gameplay))
            .add_system(update_level_num.run_in_state(GameState::Preamble))
            .add_system(death_screen.run_in_state(GameState::Gameplay))
            .add_system(
                ui_action
                    .run_not_in_state(GameState::AssetLoading)
                    .after(TrackPreviousComponent),
            );
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
pub struct LevelNum;

fn spawn_level_num(mut commands: Commands, asset_holder: Res<AssetHolder>) {
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Percent(5.),
                    right: Val::Percent(5.),
                    ..default()
                },
                ..default()
            },
            text: Text::from_section(
                "LevelNum",
                TextStyle {
                    font: asset_holder.font.clone(),
                    font_size: 64.,
                    color: Color::BLACK,
                },
            ),
            ..default()
        })
        .insert(LevelNum);
}

fn update_level_num(
    mut query: Query<&mut Text, With<LevelNum>>,
    asset_holder: Res<AssetHolder>,
    level_selection: Res<LevelSelection>,
    mut level_events: EventReader<LevelEvent>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
) {
    for event in level_events.iter() {
        if let LevelEvent::Transformed(_) = event {
            for mut text in query.iter_mut() {
                if let LevelSelection::Index(current_level_num) = *level_selection {
                    if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
                        let total_levels = ldtk_asset.iter_levels().count();
                        *text = Text::from_section(
                            format!("Level {}/{}", current_level_num + 1, total_levels),
                            TextStyle {
                                font: asset_holder.font.clone(),
                                font_size: 64.,
                                color: Color::BLACK,
                            },
                        );
                    }
                }
            }
        }
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

/// All possible actions that can be triggered by the UI.
///
/// This acts as both a component and an event.
/// Insert it on a button to define what action that button performs.
/// Then, when that button is pressed, an event of the same value will be fired.
#[allow(dead_code)]
#[derive(Clone, Eq, PartialEq, Debug, Component)]
pub enum UiAction {
    Debug(&'static str),
    SelectLevel(usize),
}

/// System that detects button presses and fires [UiAction]s.
pub(super) fn ui_action(
    actions: Query<
        (&UiAction, &Interaction, &PreviousComponent<Interaction>),
        Changed<Interaction>,
    >,
    mut event_writer: EventWriter<UiAction>,
) {
    for (action, interaction, previous) in actions.iter() {
        if (Interaction::Hovered, Interaction::Clicked) == (*interaction, *previous.get()) {
            event_writer.send(action.clone())
        }
    }
}
