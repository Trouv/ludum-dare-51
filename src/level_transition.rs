use crate::*;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct LevelTransitionPlugin;

impl Plugin for LevelTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::SpawnWorld, spawn_first_level)
            .add_enter_system(GameState::Preamble, spawn_preamble_card)
            .add_system(update_preamble_card.run_in_state(GameState::Preamble))
            .add_system(enter_to_continue.run_in_state(GameState::Preamble))
            .add_exit_system(GameState::Preamble, despawn_preamble_card)
            .add_event::<LevelStart>();
    }
}

fn spawn_first_level(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    query: Query<Entity, With<Handle<LdtkAsset>>>,
) {
    if query.is_empty() {
        commands.spawn_bundle(LdtkWorldBundle {
            ldtk_handle: asset_holder.ldtk.clone(),
            ..default()
        });
    }

    commands.insert_resource(NextState(GameState::Preamble));
}

pub struct LevelStart;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct PreambleCard;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct PreambleText(usize);

fn spawn_preamble_card(
    mut commands: Commands,
    asset_holder: Res<AssetHolder>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    level_selection: Res<LevelSelection>,
) {
    let final_card = ldtk_assets
        .get(&asset_holder.ldtk)
        .map(|l| l.get_level(&level_selection))
        .flatten()
        .is_none();

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
            color: UiColor(Color::BLACK),
            ..default()
        })
        .insert(PreambleCard)
        .with_children(|builder| {
            let text = if final_card {
                "Malcolm escaped earth!\n\nThank you for playing\n\nMade by Trouv"
            } else {
                ""
            };

            builder
                .spawn_bundle(TextBundle {
                    text: Text::from_section(
                        text,
                        TextStyle {
                            font: asset_holder.font.clone(),
                            font_size: 64.,
                            color: Color::WHITE,
                        },
                    ),
                    ..default()
                })
                .insert(PreambleText(0));

            if !final_card {
                builder.spawn_bundle(TextBundle {
                    text: Text::from_section(
                        "Press ENTER to continue..",
                        TextStyle {
                            font: asset_holder.font.clone(),
                            font_size: 64.,
                            color: Color::WHITE,
                        },
                    ),
                    style: Style {
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            bottom: Val::Percent(5.),
                            right: Val::Percent(5.),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                });
            }
        });
}

fn update_preamble_card(
    mut text_query: Query<(&mut Text, &PreambleText), Changed<PreambleText>>,
    asset_holder: Res<AssetHolder>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    level_selection: Res<LevelSelection>,
) {
    for (mut text, index) in text_query.iter_mut() {
        if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
            if let Some(level) = ldtk_asset.get_level(&level_selection) {
                if let Some(FieldValue::Strings(preambles)) = level
                    .field_instances
                    .iter()
                    .find(|f| f.identifier == "Preamble")
                    .map(|f| &f.value)
                {
                    if let LevelSelection::Index(level_num) = *level_selection {
                        let preamble = match preambles.get(index.0) {
                            Some(Some(s)) => s.clone(),
                            _ => format!("#{}", level_num),
                        };

                        *text = Text::from_section(
                            preamble,
                            TextStyle {
                                font: asset_holder.font.clone(),
                                font_size: 64.,
                                color: Color::WHITE,
                            },
                        );
                    }
                }
            }
        }
    }
}

fn enter_to_continue(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut text_query: Query<&mut PreambleText>,
    asset_holder: Res<AssetHolder>,
    ldtk_assets: Res<Assets<LdtkAsset>>,
    level_selection: Res<LevelSelection>,
    mut level_start_events: EventWriter<LevelStart>,
) {
    if input.just_pressed(KeyCode::Return) {
        for mut text in text_query.iter_mut() {
            if let Some(ldtk_asset) = ldtk_assets.get(&asset_holder.ldtk) {
                if let Some(level) = ldtk_asset.get_level(&level_selection) {
                    if let Some(FieldValue::Strings(preambles)) = level
                        .field_instances
                        .iter()
                        .find(|f| f.identifier == "Preamble")
                        .map(|f| &f.value)
                    {
                        if preambles.len() > text.0 + 1 {
                            text.0 += 1;
                        } else {
                            commands.insert_resource(NextState(GameState::Gameplay));
                            level_start_events.send(LevelStart);
                        }
                    }
                }
            }
        }
    }
}

fn despawn_preamble_card(mut commands: Commands, query: Query<Entity, With<PreambleCard>>) {
    for card_entity in query.iter() {
        commands.entity(card_entity).despawn_recursive();
    }
}
