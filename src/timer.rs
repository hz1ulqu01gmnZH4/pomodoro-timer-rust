use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

pub struct PomodoroTimer {
    work_duration: Duration,
    short_break_duration: Duration,
    long_break_duration: Duration,
    current_session: SessionType,
    session_count: u32,
    time_remaining: Duration,
    is_running: bool,
    last_update: Option<Instant>,
    total_duration: Duration,
    just_completed: bool,
}

impl PomodoroTimer {
    pub fn new() -> Self {
        let work_duration = Duration::from_secs(25 * 60);
        Self {
            work_duration,
            short_break_duration: Duration::from_secs(5 * 60),
            long_break_duration: Duration::from_secs(15 * 60),
            current_session: SessionType::Work,
            session_count: 1,
            time_remaining: work_duration,
            is_running: false,
            last_update: None,
            total_duration: work_duration,
            just_completed: false,
        }
    }

    pub fn update_durations(&mut self, work_min: u32, short_break_min: u32, long_break_min: u32) {
        self.work_duration = Duration::from_secs(work_min as u64 * 60);
        self.short_break_duration = Duration::from_secs(short_break_min as u64 * 60);
        self.long_break_duration = Duration::from_secs(long_break_min as u64 * 60);
        
        if !self.is_running {
            match self.current_session {
                SessionType::Work => {
                    self.time_remaining = self.work_duration;
                    self.total_duration = self.work_duration;
                }
                SessionType::ShortBreak => {
                    self.time_remaining = self.short_break_duration;
                    self.total_duration = self.short_break_duration;
                }
                SessionType::LongBreak => {
                    self.time_remaining = self.long_break_duration;
                    self.total_duration = self.long_break_duration;
                }
            }
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.last_update = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        self.is_running = false;
        self.last_update = None;
    }

    pub fn reset(&mut self) {
        self.is_running = false;
        self.last_update = None;
        match self.current_session {
            SessionType::Work => self.time_remaining = self.work_duration,
            SessionType::ShortBreak => self.time_remaining = self.short_break_duration,
            SessionType::LongBreak => self.time_remaining = self.long_break_duration,
        }
    }

    pub fn skip(&mut self) {
        // Save the current running state before skipping
        let was_running = self.is_running;
        self.complete_session();
        // If timer wasn't running before skip, stop it after transitioning
        if !was_running {
            self.is_running = false;
            self.last_update = None;
        }
    }

    pub fn update(&mut self) {
        if self.is_running {
            if let Some(last_update) = self.last_update {
                let now = Instant::now();
                let elapsed = now - last_update;
                
                if elapsed >= self.time_remaining {
                    self.time_remaining = Duration::ZERO;
                    self.complete_session();
                } else {
                    self.time_remaining -= elapsed;
                }
                
                self.last_update = Some(now);
            }
        }
    }

    fn complete_session(&mut self) {
        self.just_completed = true;

        match self.current_session {
            SessionType::Work => {
                if self.session_count % 4 == 0 {
                    self.current_session = SessionType::LongBreak;
                    self.time_remaining = self.long_break_duration;
                    self.total_duration = self.long_break_duration;
                } else {
                    self.current_session = SessionType::ShortBreak;
                    self.time_remaining = self.short_break_duration;
                    self.total_duration = self.short_break_duration;
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.current_session = SessionType::Work;
                self.time_remaining = self.work_duration;
                self.total_duration = self.work_duration;
                if self.session_count % 4 == 0 {
                    self.session_count = 1;
                } else {
                    self.session_count += 1;
                }
            }
        }
        
        // Automatically start the next session
        self.is_running = true;
        self.last_update = Some(Instant::now());
    }

    pub fn get_time_string(&mut self) -> String {
        self.update();
        let total_seconds = self.time_remaining.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub fn get_progress(&mut self) -> f32 {
        self.update();
        let elapsed = self.total_duration - self.time_remaining;
        elapsed.as_secs_f32() / self.total_duration.as_secs_f32()
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn get_session_type(&self) -> SessionType {
        self.current_session
    }

    pub fn get_session_count(&self) -> u32 {
        match self.current_session {
            SessionType::Work => self.session_count,
            _ => if self.session_count == 1 { 4 } else { self.session_count - 1 }
        }
    }

    pub fn just_completed(&self) -> bool {
        self.just_completed
    }

    pub fn clear_completed_flag(&mut self) {
        self.just_completed = false;
    }
}