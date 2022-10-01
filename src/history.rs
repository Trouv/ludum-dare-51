use crate::GameState;
use bevy::prelude::*;
use iyes_loopless::prelude::*;
use std::time::Duration;

use crate::event_scheduler::{EventScheduler, EventSchedulerPlugin};

pub struct HistoryPlugin;

impl Plugin for HistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EventSchedulerPlugin::<RewindEvent>::new())
            .insert_resource(TimeScale(1.))
            .insert_resource(TimeSinceLevelStart(0.))
            .add_system(rewind.run_in_state(GameState::Gameplay))
            .add_system(
                stop_rewind
                    .run_on_event::<RewindEvent>()
                    .run_in_state(GameState::Gameplay),
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

pub enum RewindEvent {
    Start,
    Stop,
}

pub fn rewind(
    input: Res<Input<KeyCode>>,
    mut time_scale: ResMut<TimeScale>,
    mut rewind_event_scheduler: ResMut<EventScheduler<RewindEvent>>,
) {
    if input.just_pressed(KeyCode::Z) && time_scale.0 >= 0. {
        *time_scale = TimeScale(-10.);
        rewind_event_scheduler.schedule(RewindEvent::Start, Duration::ZERO);
        rewind_event_scheduler.schedule(RewindEvent::Stop, Duration::from_millis(1000));
    }
}

pub fn stop_rewind(mut rewind_events: EventReader<RewindEvent>, mut time_scale: ResMut<TimeScale>) {
    for e in rewind_events.iter() {
        match e {
            RewindEvent::Stop => {
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
) {
    time_since_level_start.0 = time_since_level_start.0 + bevy_time.delta_seconds() * time_scale.0;

    if time_since_level_start.0 < 0. {
        time_since_level_start.0 = 0.;
        time_scale.0 = 0.;
    }
}
