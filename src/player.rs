use crate::GameState;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use std::collections::HashSet;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash, Component)]
pub struct Player;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Hash)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(movement.run_in_state(GameState::Gameplay))
            .add_system(spawn_ground_sensor.run_in_state(GameState::Gameplay))
            .add_system(ground_detection.run_in_state(GameState::Gameplay))
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
    #[sprite_bundle("player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    pub ground_detection: GroundDetection,
}

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::cuboid(6., 14.),
                rigid_body: RigidBody::Dynamic,
                locked_axes: LockedAxes::ROTATION_LOCKED,
                friction: Friction {
                    coefficient: 0.1,
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
    mut query: Query<(&mut Velocity, &GroundDetection), With<Player>>,
) {
    for (mut velocity, ground_detection) in query.iter_mut() {
        let right = if input.pressed(KeyCode::D) { 1. } else { 0. };
        let left = if input.pressed(KeyCode::A) { 1. } else { 0. };

        velocity.linvel.x = (right - left) * 200.;

        if input.just_pressed(KeyCode::Space) && (ground_detection.on_ground) {
            velocity.linvel.y = 450.;
        } else {
            velocity.linvel.y -= 10.;
        }
    }
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Transform), Added<GroundDetection>>,
) {
    for (entity, transform) in detect_ground_for.iter() {
        let detector_shape = Collider::cuboid(3., 2.);

        let sensor_translation = Vec3::new(0., -14., 0.) / transform.scale;

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
