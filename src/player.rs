use crate::{animation::*, from_component::FromComponentPlugin, GameState};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use std::collections::HashSet;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct PlayerPlugin;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Component, Default)]
pub enum PlayerAnimationState {
    #[default]
    Idle,
    Running,
    Falling,
}

impl From<PlayerAnimationState> for SpriteSheetAnimation {
    fn from(animation_state: PlayerAnimationState) -> Self {
        let indices = match animation_state {
            PlayerAnimationState::Idle => 0..1,
            PlayerAnimationState::Running => 4..8,
            PlayerAnimationState::Falling => 8..9,
        };

        let frame_timer = Timer::from_seconds(0.2, true);

        let repeat = true;

        SpriteSheetAnimation {
            indices,
            frame_timer,
            repeat,
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement.run_in_state(GameState::Gameplay).label("movement"))
            .add_system(spawn_ground_sensor.run_in_state(GameState::Gameplay))
            .add_system(ground_detection.run_in_state(GameState::Gameplay))
            .add_plugin(FromComponentPlugin::<
                PlayerAnimationState,
                SpriteSheetAnimation,
            >::new())
            .add_system(
                move_object_with_ground
                    .run_in_state(GameState::Gameplay)
                    .after("movement"),
            )
            //.add_system(
            //|mut collision_events: EventReader<CollisionEvent>,
            //mut contact_force_events: EventReader<ContactForceEvent>| {
            //for collision_event in collision_events.iter() {
            //println!("Received collision event: {:?}", collision_event);
            //}
            //for contact_force_event in contact_force_events.iter() {
            //println!("Received contact force event: {:?}", contact_force_event);
            //}
            //},
            //)
            //.add_system(|query: Query<&GroundSensor, Changed<GroundSensor>>| {
            //query.for_each(|gs| {
            //dbg!(gs);
            //});
            //})
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}

#[derive(Component, Clone, Eq, PartialEq, Debug)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}

#[derive(Clone, Default, Component)]
pub struct GroundDetection {
    pub on_ground: bool,
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle]
    #[bundle]
    pub sprite_sheet_bundle: SpriteSheetBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    pub ground_detection: GroundDetection,
    pub animation: PlayerAnimationState,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub restitution: Restitution,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(10., 24.),
                rigid_body: RigidBody::Dynamic,
                locked_axes: LockedAxes::ROTATION_LOCKED,
                friction: Friction {
                    coefficient: 0.1,
                    combine_rule: CoefficientCombineRule::Multiply,
                },
                restitution: Restitution {
                    coefficient: 0.,
                    combine_rule: CoefficientCombineRule::Multiply,
                },
                ..Default::default()
            },
            "Platform" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::KinematicVelocityBased,
                locked_axes: LockedAxes::ROTATION_LOCKED,
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

pub fn movement(
    input: Res<Input<KeyCode>>,
    mut query: Query<
        (
            &mut Velocity,
            &mut PlayerAnimationState,
            &mut TextureAtlasSprite,
            &GroundDetection,
        ),
        With<Player>,
    >,
    time: Res<Time>,
    mut x_velocity_contribution: Local<f32>,
) {
    for (mut velocity, mut animation_state, mut sprite, ground_detection) in query.iter_mut() {
        let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
        let left = if input.pressed(KeyCode::A) { 1. } else { 0. };

        let max_contribution = 250.;
        if ground_detection.on_ground {
            let goal = (right - left) * max_contribution;
            *x_velocity_contribution +=
                (goal - *x_velocity_contribution) / 2. * time.delta_seconds() * 70.;

            if goal.abs() > 0. {
                if *animation_state != PlayerAnimationState::Running {
                    *animation_state = PlayerAnimationState::Running;
                }
            } else {
                if *animation_state != PlayerAnimationState::Idle {
                    *animation_state = PlayerAnimationState::Idle;
                }
            }

            if goal > 0. {
                sprite.flip_x = false;
            } else if goal < 0. {
                sprite.flip_x = true;
            }

            velocity.linvel.x = *x_velocity_contribution;
        } else {
            if *animation_state != PlayerAnimationState::Falling {
                *animation_state = PlayerAnimationState::Falling;
            }
            let contribution = (right - left) * 1200. * time.delta_seconds();

            if contribution < 0. && velocity.linvel.x > -max_contribution {
                velocity.linvel.x += contribution;
            } else if contribution > 0. && velocity.linvel.x < max_contribution {
                velocity.linvel.x += contribution;
            }

            *x_velocity_contribution = velocity.linvel.x;
        }

        if input.just_pressed(KeyCode::Space) && (ground_detection.on_ground) {
            velocity.linvel.y = velocity.linvel.y.max(0.) + 400.;
        } else if input.pressed(KeyCode::Space) {
            velocity.linvel.y -= 900. * time.delta_seconds();
        } else {
            velocity.linvel.y -= 1200. * time.delta_seconds();
        }
    }
}

pub fn move_object_with_ground(
    mut detectors: Query<(Entity, &mut Velocity, &GroundDetection)>,
    sensors: Query<&GroundSensor>,
    velocities: Query<&Velocity, Without<GroundDetection>>,
) {
    for (detector_entity, mut detect_velocity, detector) in detectors.iter_mut() {
        if detector.on_ground {
            if let Some(sensor) = sensors
                .iter()
                .find(|s| s.ground_detection_entity == detector_entity)
            {
                // what am I standing on?
                for ground_entity in sensor.intersecting_ground_entities.iter() {
                    if let Ok(velocity) = velocities.get(*ground_entity) {
                        detect_velocity.linvel.x += velocity.linvel.x;
                    }
                }
            }
        }
    }
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Transform), Added<GroundDetection>>,
) {
    for (entity, transform) in detect_ground_for.iter() {
        let detector_shape = Collider::cuboid(9., 2.);

        let sensor_translation = Vec3::new(0., -24., 0.) / transform.scale;

        commands.entity(entity).with_children(|builder| {
            builder
                .spawn()
                .insert(detector_shape)
                .insert(Sensor)
                .insert(Transform::from_translation(sensor_translation))
                .insert(GlobalTransform::default())
                .insert(ActiveEvents::COLLISION_EVENTS)
                .insert(GroundSensor {
                    ground_detection_entity: entity,
                    intersecting_ground_entities: HashSet::new(),
                });
        });
    }
}

pub fn ground_detection(
    mut ground_detectors: Query<&mut GroundDetection>,
    mut ground_sensors: Query<(Entity, &mut GroundSensor)>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<Entity, (With<Collider>, Without<Sensor>)>,
) {
    for (entity, mut ground_sensor) in ground_sensors.iter_mut() {
        for collision in collisions.iter() {
            match collision {
                CollisionEvent::Started(a, b, _) => {
                    let (sensor, other) = if *a == entity {
                        (a, b)
                    } else if *b == entity {
                        (b, a)
                    } else {
                        continue;
                    };

                    if collidables.contains(*other) {
                        if *sensor == entity {
                            ground_sensor.intersecting_ground_entities.insert(*other);
                        }
                    }
                }
                CollisionEvent::Stopped(a, b, _) => {
                    let (sensor, other) = if *a == entity {
                        (a, b)
                    } else if *b == entity {
                        (b, a)
                    } else {
                        continue;
                    };

                    if *sensor == entity {
                        ground_sensor.intersecting_ground_entities.remove(other);
                    }
                }
            }
        }

        if let Ok(mut ground_detection) =
            ground_detectors.get_mut(ground_sensor.ground_detection_entity)
        {
            ground_detection.on_ground = ground_sensor.intersecting_ground_entities.len() > 0;
        }
    }
}
