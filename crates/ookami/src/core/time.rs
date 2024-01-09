use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};
use tracing::*;

#[derive(Debug)]
pub struct Time {
    tick_rate: f64,
    tick_time: Duration,
    lag_time: Duration,
    step_count: u32,
    bail_threshold: u32,
    start_time: Instant,
    previous_frame: Instant,
    current_frame: Instant,
    delta_time: Duration,
    tick_previous_frame: Instant,
    tick_current_frame: Instant,
    tick_delta_time: Duration,
    past_delta_times: arraydeque::ArrayDeque<Duration, 1024>,
    average_delta_time: Duration,
}

#[allow(unused)]
impl Time {
    pub fn new(tick_rate: f64, bail_threshold: u32) -> Self {
        Self {
            tick_rate,
            tick_time: Duration::from_secs_f64(1. / tick_rate),
            lag_time: Default::default(),
            step_count: 0,
            bail_threshold,
            start_time: Instant::now(),
            previous_frame: Instant::now(),
            current_frame: Instant::now(),
            delta_time: Default::default(),
            tick_previous_frame: Instant::now(),
            tick_current_frame: Instant::now(),
            tick_delta_time: Default::default(),
            past_delta_times: Default::default(),
            average_delta_time: Default::default(),
        }
    }

    pub fn tick_rate(&self) -> f64 {
        self.tick_rate
    }

    pub fn tick_time(&self) -> &Duration {
        &self.tick_time
    }

    pub fn delta(&self) -> &Duration {
        &self.delta_time
    }

    pub fn delta_secs(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }

    pub fn average_delta(&self) -> &Duration {
        &self.average_delta_time
    }

    pub fn average_delta_secs(&self) -> f64 {
        self.average_delta_time.as_secs_f64()
    }

    pub fn delta_tick(&self) -> &Duration {
        &self.tick_delta_time
    }

    pub fn delta_tick_secs(&self) -> f64 {
        self.tick_delta_time.as_secs_f64()
    }

    pub fn now(&self) -> Instant {
        Instant::now()
    }

    pub(crate) fn next_tick(&mut self) -> bool {
        self.update();
        self.should_do_tick() && {
            self.tick();
            true
        }
    }

    pub(crate) fn update(&mut self) {
        self.current_frame = Instant::now();
        self.delta_time = self.current_frame - self.previous_frame;
        self.previous_frame = self.current_frame;
        self.lag_time += self.delta_time;
        self.step_count = 0;
        if let Err(arraydeque::CapacityError { element }) =
            self.past_delta_times.push_back(self.delta_time)
        {
            self.past_delta_times.pop_front();
            self.past_delta_times.push_back(element);
        }
        self.average_delta_time = {
            let sum: Duration = self.past_delta_times.iter().sum();
            sum.div_f64(self.past_delta_times.len() as f64)
        }
    }

    pub(crate) fn tick(&mut self) {
        self.tick_current_frame = Instant::now();
        self.tick_delta_time = self.tick_current_frame - self.tick_previous_frame;
        self.tick_previous_frame = self.tick_current_frame;
        self.lag_time -= self.tick_time;
        self.step_count += 1;
    }

    pub(crate) fn should_do_tick(&self) -> bool {
        if self.step_count >= self.bail_threshold {
            warn!("Struggling to catch up with tick rate.");
        }
        self.lag_time >= self.tick_time && self.step_count < self.bail_threshold
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new(128., 1024)
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[tick_rate: {:?}, delta_tick: {:?}, delta: {:?}, time_since_start: {:?}, now: {:?}]",
            self.tick_rate,
            self.tick_delta_time,
            self.delta_time,
            self.now().duration_since(self.start_time),
            self.now(),
        )
    }
}

pub struct GameLoop {
    pub time: Time,
    pub start: Box<dyn FnMut(&Time)>,
    pub early_update: Box<dyn FnMut(&Time)>,
    pub fixed_update: Box<dyn FnMut(&Time)>,
    pub update: Box<dyn FnMut(&Time)>,
    pub stop: Box<dyn FnMut(&Time)>,
}

impl Default for GameLoop {
    fn default() -> Self {
        Self {
            time: Time::new(128.0, 1024),
            start: Box::new(|_| {}),
            early_update: Box::new(|_| {}),
            fixed_update: Box::new(|_| {}),
            update: Box::new(|_| {}),
            stop: Box::new(|_| {}),
        }
    }
}

#[allow(unused)]
impl GameLoop {
    pub fn run(mut self, should_continue: impl Fn() -> bool) {
        (self.start)(&self.time);
        while (should_continue)() {
            self.time.update();
            (self.early_update)(&self.time);
            while self.time.should_do_tick() {
                self.time.tick();
                (self.fixed_update)(&self.time);
            }
            (self.update)(&self.time);
        }
        (self.stop)(&self.time);
    }
}
