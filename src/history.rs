use crate::{level_transition::LevelStart, player::Vitality, GameState};
use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkLevel, Respawn};
use iyes_loopless::prelude::*;
use std::time::Duration;

use crate::event_scheduler::{EventScheduler, EventSchedulerPlugin};

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventSchedulerPlugin::<TimeEvent>::new())
            .insert_resource(TimeScale(1.))
            .insert_resource(TimeSinceLevelStart(0.))
            .add_system(rewind.run_in_state(GameState::Gameplay))
            .add_system(
                stop_rewind
                    .run_on_event::<TimeEvent>()
                    .run_in_state(GameState::Gameplay)
                    .label("update_time"),
            )
            //.add_system(|time_since_level_start: Res<TimeSinceLevelStart>| {
            //dbg!(time_since_level_start);
            //})
            .add_system(
                update_time
                    .run_in_state(GameState::Gameplay)
                    .label("update_time"),
            );
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct Moment<T> {
    pub timestamp: f32,
    pub data: T,
}

#[derive(Clone, PartialEq, Debug, Default, Component)]
pub struct History<T> {
    pub moments: Vec<Moment<T>>,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct TimeSinceLevelStart(pub f32);

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct TimeScale(pub f32);

pub enum TimeEvent {
    Rewind,
    FastForward,
    Normal,
}

pub fn rewind(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut time_scale: ResMut<TimeScale>,
    mut rewind_event_scheduler: ResMut<EventScheduler<TimeEvent>>,
    vitality: Query<&Vitality>,
    levels: Query<Entity, With<Handle<LdtkLevel>>>,
    mut level_start_events: EventWriter<LevelStart>,
) {
    if input.just_pressed(KeyCode::Z) && time_scale.0 == 1. && *vitality.single() == Vitality::Alive
    {
        *time_scale = TimeScale(-10.);
        rewind_event_scheduler.schedule(TimeEvent::Rewind, Duration::ZERO);
        rewind_event_scheduler.schedule(TimeEvent::Normal, Duration::from_millis(200));
    } else if input.just_pressed(KeyCode::X)
        && time_scale.0 == 1.
        && *vitality.single() == Vitality::Alive
    {
        *time_scale = TimeScale(10.);
        rewind_event_scheduler.schedule(TimeEvent::FastForward, Duration::ZERO);
        rewind_event_scheduler.schedule(TimeEvent::Normal, Duration::from_millis(200));
    }

    if input.just_pressed(KeyCode::R) {
        commands.entity(levels.single()).insert(Respawn);
        level_start_events.send(LevelStart);
    }
}

pub fn stop_rewind(mut rewind_events: EventReader<TimeEvent>, mut time_scale: ResMut<TimeScale>) {
    for e in rewind_events.iter() {
        match e {
            TimeEvent::Normal => {
                *time_scale = TimeScale(1.);
            }
            _ => (),
        }
    }
}

pub fn update_time(
    mut time_scale: ResMut<TimeScale>,
    mut time_since_level_start: ResMut<TimeSinceLevelStart>,
    bevy_time: Res<Time>,
    mut level_events: EventReader<LevelStart>,
    mut time_events: EventWriter<TimeEvent>,
    mut vitals: Query<&mut Vitality>,
) {
    for _ in level_events.iter() {
        time_scale.0 = 1.;
        time_since_level_start.0 = 0.;
        time_events.send(TimeEvent::Normal);
        for mut vitality in vitals.iter_mut() {
            *vitality = Vitality::Alive;
        }
    }

    time_since_level_start.0 = time_since_level_start.0 + bevy_time.delta_seconds() * time_scale.0;

    if time_since_level_start.0 < 0. {
        time_since_level_start.0 = 0.;
        time_scale.0 = 0.;
    }
    if time_since_level_start.0 > 10. {
        time_since_level_start.0 = 10.;
        time_scale.0 = 0.;
        for mut vitality in vitals.iter_mut() {
            *vitality = Vitality::Dead;
        }
    }
}
