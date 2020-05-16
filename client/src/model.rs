use hop_engine::metrics::Metric;
use std::{collections::HashMap, convert::TryInto};

pub struct StatsData {
    inner: HashMap<Vec<u8>, Vec<u8>>,
}

impl StatsData {
    pub(crate) fn new(map: HashMap<Vec<u8>, Vec<u8>>) -> Self {
        Self { inner: map }
    }

    fn int(&self, metric: Metric) -> i64 {
        self.inner
            .get(metric.name().as_bytes())
            .and_then(|bytes| bytes.as_slice().try_into().ok())
            .map(i64::from_be_bytes)
            .unwrap_or_default()
    }

    pub fn commands_errored(&self) -> i64 {
        self.int(Metric::CommandsErrored)
    }

    pub fn commands_successful(&self) -> i64 {
        self.int(Metric::CommandsSuccessful)
    }

    pub fn sessions_ended(&self) -> i64 {
        self.int(Metric::SessionsEnded)
    }

    pub fn sessions_started(&self) -> i64 {
        self.int(Metric::SessionsStarted)
    }
}
