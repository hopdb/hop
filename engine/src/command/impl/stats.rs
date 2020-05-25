use super::super::{response, Dispatch, DispatchError, DispatchResult, Request};
use crate::metrics::Metric;
use crate::Hop;
use alloc::vec::Vec;
use core::convert::TryFrom;
use dashmap::DashMap;

pub struct Stats;

impl Stats {
    const COUNTERS: &'static [Metric] = &[
        Metric::CommandsErrored,
        Metric::CommandsSuccessful,
        Metric::SessionsStarted,
    ];
}

impl Dispatch for Stats {
    fn dispatch(hop: &Hop, req: &Request, res: &mut Vec<u8>) -> DispatchResult<()> {
        if req.key_type().is_some() {
            return Err(DispatchError::KeyTypeUnexpected);
        }

        let map = DashMap::with_capacity(4);
        let metrics = hop.metrics();

        for counter in Self::COUNTERS {
            let count = match metrics.counter(counter).and_then(|x| i64::try_from(x).ok()) {
                Some(count) => count,
                None => continue,
            };
            let key = counter.name().as_bytes().to_vec();
            let value = count.to_be_bytes().to_vec();

            map.insert(key, value);
        }

        response::write_map(res, &map);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;
    use crate::{
        command::{request::RequestBuilder, CommandId, Dispatch, DispatchError, Response},
        metrics::Metric,
        state::KeyType,
        Hop,
    };
    use dashmap::DashMap;

    #[test]
    fn test_stats_empty() {
        let req = RequestBuilder::new(CommandId::Stats).into_request();

        let hop = Hop::new();
        let mut resp = Vec::new();

        assert!(Stats::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(DashMap::new()).as_bytes());
    }

    #[test]
    fn test_stats_with_entry() {
        let req = RequestBuilder::new(CommandId::Stats).into_request();

        let hop = Hop::new();
        hop.0.metrics_writer.increment(Metric::CommandsSuccessful);

        let mut resp = Vec::new();
        let expected = DashMap::new();
        expected.insert(b"commands_successful".to_vec(), 1i64.to_be_bytes().to_vec());

        assert!(Stats::dispatch(&hop, &req, &mut resp).is_ok());
        assert_eq!(resp, Response::from(expected).as_bytes());
    }

    #[test]
    fn test_stats_errors_with_key_type() {
        let mut builder = RequestBuilder::new(CommandId::Stats);
        builder.key_type(KeyType::Map);
        let req = builder.into_request();

        let hop = Hop::new();
        let mut resp = Vec::new();

        assert_eq!(
            Stats::dispatch(&hop, &req, &mut resp).unwrap_err(),
            DispatchError::KeyTypeUnexpected
        );
    }
}
