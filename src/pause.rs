use crate::{
    player::Vitality, previous_component::PreviousComponent, ui::UiAction, AssetHolder, GameState,
};
use bevy::{prelude::*, ui::FocusPolicy};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(pause.run_in_state(GameState::Gameplay))
            .add_system(unpause.run_in_state(GameState::PauseMenu))
            .add_system(level_select.run_in_state(GameState::PauseMenu))
            .add_enter_system(GameState::PauseMenu, spawn_pause_screen)
            .add_exit_system(GameState::PauseMenu, despawn_pause_menu);
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
struct PauseScreen;

fn pause(mut commands: Commands, input: Res<Input<KeyCode>>, vitality: Query<&Vitality>) {
    if (input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::P))
        && *vitality.single() == Vitality::Alive
    {
        commands.insert_resource(NextState(GameState::PauseMenu))
    }
}

fn unpause(mut commands: Commands, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) || input.just_pressed(KeyCode::P) {
        commands.insert_resource(NextState(GameState::Gameplay))
    }
}

fn spawn_pause_screen(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_config.timestep_mode = TimestepMode::Variable {
        max_dt: 1.0 / 60.0,
        time_scale: 0.0,
        substeps: 1,
    };

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
        .insert(PauseScreen)
        .with_children(|builder| {
            builder.spawn_bundle(TextBundle {
                style: Style {
                    position: UiRect {
                        top: Val::Percent(5.),
                        left: Val::Percent(5.),
                        ..default()
                    },
                    position_type: PositionType::Absolute,
                    ..default()
                },
                text: Text::from_section(
                    "Level Select",
                    TextStyle {
                        font: asset_holder.font.clone(),
                        font_size: 128.,
                        color: Color::WHITE,
                    },
                ),
                ..default()
            });

            for (level_num, _) in ldtk_assets
                .get(&asset_holder.ldtk)
                .unwrap()
                .iter_levels()
                .enumerate()
            {
                builder
                    .spawn_bundle(ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            align_content: AlignContent::Center,
                            flex_wrap: FlexWrap::Wrap,
                            margin: UiRect::new(
                                Val::Px(10.),
                                Val::Px(10.),
                                Val::Px(10.),
                                Val::Px(10.),
                            ),
                            ..default()
                        },
                        color: UiColor(Color::WHITE),
                        ..default()
                    })
                    .insert(UiAction::SelectLevel(level_num))
                    .insert(PreviousComponent::<Interaction>::default())
                    .with_children(|button| {
                        button.spawn_bundle(TextBundle {
                            style: Style {
                                margin: UiRect::new(
                                    Val::Px(10.),
                                    Val::Px(10.),
                                    Val::Px(10.),
                                    Val::Px(10.),
                                ),
                                ..default()
                            },
                            text: Text::from_section(
                                format!("#{}", level_num + 1),
                                TextStyle {
                                    font: asset_holder.font.clone(),
                                    font_size: 64.,
                                    color: Color::BLACK,
                                },
                            ),
                            focus_policy: FocusPolicy::Pass,
                            ..default()
                        });
                    });
            }
        });
}

fn despawn_pause_menu(
    mut commands: Commands,
    pause_screen: Query<Entity, With<PauseScreen>>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_config.timestep_mode = TimestepMode::Variable {
        max_dt: 1.0 / 60.0,
        time_scale: 1.0,
        substeps: 1,
    };
    commands.entity(pause_screen.single()).despawn_recursive();
}

fn level_select(
    mut commands: Commands,
    mut level_selection: ResMut<LevelSelection>,
    mut ui_actions: EventReader<UiAction>,
    levels: Query<Entity, With<Handle<LdtkLevel>>>,
) {
    for action in ui_actions.iter() {
        match action {
            UiAction::SelectLevel(num) => {
                if let LevelSelection::Index(old_num) = *level_selection {
                    if old_num == *num {
                        commands.entity(levels.single()).insert(Respawn);
                    }
                }

                *level_selection = LevelSelection::Index(*num);
                commands.insert_resource(NextState(GameState::Preamble));
            }
            _ => (),
        }
    }
}
