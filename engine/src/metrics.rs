use hop_internal_metrics::{Metrics as BaseMetrics, Reader as BaseReader, Writer as BaseWriter};

pub type Metrics = BaseMetrics<Metric>;
pub type Reader = BaseReader<Metric>;
pub type Writer = BaseWriter<Metric>;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Metric {
    CommandsSuccessful = 0,
    CommandsErrored = 1,
    SessionsStarted = 10,
    SessionsEnded = 11,
}

impl Metric {
    pub fn name(self) -> &'static str {
        match self {
            Metric::CommandsErrored => "commands_errored",
            Metric::CommandsSuccessful => "commands_successful",
            Metric::SessionsEnded => "sessions_ended",
            Metric::SessionsStarted => "sessions_started",
        }
    }
}
