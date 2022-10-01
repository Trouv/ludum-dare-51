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
            .add_system(platform_movement.run_in_state(GameState::Gameplay));
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
        &Transform,
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
                * path.speed
                * time_scale.0;

            if new_velocity.dot(velocity.linvel) <= 0. || history.moments.is_empty() {
                // passed it!
                path.index = (path.index + 1) % path.points.len();
                let goal_position = path.points[path.index];

                new_velocity = (goal_position - transform.translation)
                    .normalize_or_zero()
                    .truncate()
                    * path.speed
                    * time_scale.0;

                history.moments.push(Moment {
                    data: PlatformMoment::ChangeDirection {
                        velocity: new_velocity,
                        position: transform.translation,
                    },
                    timestamp: time_since_level_start.0,
                });
            }

            velocity.linvel = new_velocity;
        }
    } else {
        for (transform, mut path, mut history, mut velocity) in query.iter_mut() {
            if let Some(goal_moment) = history.moments.last() {
                if time_since_level_start.0 < goal_moment.timestamp {
                    history.moments.pop();

                    if path.index == 0 {
                        path.index = path.points.len() - 1;
                    } else {
                        path.index -= 1;
                    }
                }
            }

            if let Some(goal_moment) = history.moments.last() {
                match goal_moment.data {
                    PlatformMoment::ChangeDirection {
                        position: goal_position,
                        velocity: _,
                    } => {
                        velocity.linvel = (goal_position - transform.translation)
                            .truncate()
                            .normalize()
                            * path.speed
                            * time_scale.0.abs();
                    }
                }
            } else {
                velocity.linvel = Vec2::ZERO;
            }
        }
    }
}
