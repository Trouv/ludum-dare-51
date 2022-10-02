use crate::{
    history::{History, Moment, TimeScale, TimeSinceLevelStart},
    GameState,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::{prelude::*, utils::ldtk_grid_coords_to_translation_centered};
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

pub struct PlatformPlugin;

impl Plugin for PlatformPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlatformBundle>("Platform")
            //.add_system(
            //|query: Query<
            //(&History<PlatformMoment>, &Path),
            //Changed<History<PlatformMoment>>,
            //>| {
            //query.for_each(|(x, path)| {
            //dbg!(x.moments.len(), path.index);
            //});
            //},
            //)
            .add_system(
                platform_movement
                    .run_in_state(GameState::Gameplay)
                    .after("update_time"),
            );
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum PlatformMoment {
    ChangeDirection { velocity: Vec2, position: Vec3 },
}

impl Default for PlatformMoment {
    fn default() -> Self {
        PlatformMoment::ChangeDirection {
            velocity: Vec2::default(),
            position: Vec3::default(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct Path {
    points: Vec<Vec3>,
    speed: f32,
    index: usize,
}

impl LdtkEntity for Path {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlas>,
    ) -> Self {
        if let FieldValue::Float(Some(speed)) = entity_instance
            .field_instances
            .iter()
            .find(|f| f.identifier == "Speed")
            .and_then(|f| Some(&f.value))
            .expect("platform should have a speed value")
        {
            if let Some(FieldValue::Points(path_field)) = entity_instance
                .field_instances
                .iter()
                .find(|f| f.identifier == "Path")
                .and_then(|f| Some(&f.value))
            {
                let mut points = vec![ldtk_grid_coords_to_translation_centered(
                    entity_instance.grid,
                    layer_instance.c_hei,
                    IVec2::splat(layer_instance.grid_size),
                )
                .extend(0.)];

                for point in path_field.iter() {
                    let point = point.expect("path points shouldn't be null");

                    points.push(
                        ldtk_grid_coords_to_translation_centered(
                            point,
                            layer_instance.c_hei,
                            IVec2::splat(layer_instance.grid_size),
                        )
                        .extend(0.),
                    );
                }

                Path {
                    points,
                    speed: *speed,
                    index: 0,
                }
            } else {
                Path::default()
            }
        } else {
            Path::default()
        }
    }
}

#[derive(Clone, Default, LdtkEntity, Bundle)]
struct PlatformBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[ldtk_entity]
    pub path: Path,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: crate::player::ColliderBundle,
    pub history: History<PlatformMoment>,
}

fn platform_movement(
    mut query: Query<(
        &mut Transform,
        &mut Path,
        &mut History<PlatformMoment>,
        &mut Velocity,
    )>,
    time_scale: Res<TimeScale>,
    time_since_level_start: Res<TimeSinceLevelStart>,
) {
    if time_scale.0 > 0. {
        for (transform, mut path, mut history, mut velocity) in query.iter_mut() {
            let goal_position = path.points[path.index];
            let mut new_velocity = (goal_position - transform.translation)
                .truncate()
                .normalize()
                * path.speed;

            let goal_passed = match history.moments.last() {
                Some(Moment {
                    data:
                        PlatformMoment::ChangeDirection {
                            velocity: goal_velocity,
                            ..
                        },
                    ..
                }) => {
                    if new_velocity.dot(*goal_velocity) < 0. {
                        true
                    } else {
                        false
                    }
                }
                None => true,
            };

            if goal_passed {
                // passed it!
                path.index = (path.index + 1) % path.points.len();
                let goal_position = path.points[path.index];

                new_velocity = (goal_position - transform.translation)
                    .normalize_or_zero()
                    .truncate()
                    * path.speed;

                history.moments.push(Moment {
                    data: PlatformMoment::ChangeDirection {
                        velocity: new_velocity,
                        position: transform.translation,
                    },
                    timestamp: time_since_level_start.0,
                });
            }

            velocity.linvel = new_velocity * time_scale.0;
        }
    } else {
        for (mut transform, mut path, mut history, mut velocity) in query.iter_mut() {
            // Popping items off the history if we've passed them
            if let Some(goal_moment) = history.moments.last() {
                if time_since_level_start.0 < goal_moment.timestamp
                    || time_since_level_start.0 == 0.
                {
                    let PlatformMoment::ChangeDirection { position, .. } = goal_moment.data;
                    if history.moments.len() > 1 {
                        transform.translation = position;

                        history.moments.pop();

                        if path.index == 0 {
                            path.index = path.points.len() - 1;
                        } else {
                            path.index -= 1;
                        }
                    }
                }
            }

            // actually apply the history
            if let Some(goal_moment) = history.moments.last() {
                match goal_moment.data {
                    PlatformMoment::ChangeDirection {
                        position: _,
                        velocity: goal_velocity,
                    } => {
                        velocity.linvel = goal_velocity * time_scale.0;
                    }
                }
            }
        }
    }
}
