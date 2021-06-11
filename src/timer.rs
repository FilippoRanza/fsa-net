use std::time;
use std::fmt;

pub fn parse_time_spec(time_spec: &str) -> Result<u64, TimeSpecError> {
    if let Ok(time) = time_spec.parse() {
        Ok(time)
    } else{
        Err(TimeSpecError{})
    }
}

#[derive(Debug)]
pub struct TimeSpecError {}

impl fmt::Display for TimeSpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error:")
    }
}

#[derive(Default)]
pub struct TimerFactory {
    time_limit: Option<u64>
}



impl TimerFactory {
    pub fn new_timer(&self) -> Timer {
        let timer = if let Some(time_limit) = self.time_limit {
            TimerImpl::Real(RunningTimer::start(time_limit))
        } else {
            TimerImpl::Mock
        };
        Timer {timer}
    }    
    
    pub fn from_value(time_limit: Option<u64>) -> Self {
        Self {
            time_limit
        }
    }
}




pub struct Timer {
    timer: TimerImpl
}

impl Timer {
    pub fn timeout(&self) -> bool {
        self.timer.timeout()
    }
}

enum TimerImpl {
    Mock,
    Real(RunningTimer)
}

impl TimerImpl {
    fn timeout(&self) -> bool {
        match self {
            Self::Mock => false,
            Self::Real(rt) => rt.timeout()
        }
    }
}

struct RunningTimer {
    start: time::Instant,
    duration: time::Duration
}

impl RunningTimer {
    fn start(time: u64) -> Self {
        let duration = time::Duration::from_micros(time);
        let start = time::Instant::now();
        Self {
            duration,
            start
        }
    }

    fn timeout(&self) -> bool {
        self.start.elapsed() > self.duration
    }
}



#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_running_timer() {
        let factory = TimerFactory{ time_limit: Some(4)};

        let mut count = 0;
        let timer = factory.new_timer();
        while !timer.timeout() {
            count += 1;
        }
        assert!(count > 0);
    }

}