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
    fn dispatch(hop: &Hop, req: &Request) -> DispatchResult<Vec<u8>> {
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

        Ok(response::write_map(&map))
    }
}

#[cfg(test)]
mod tests {
    use super::Stats;
    use crate::{
        command::{response::ResponseType, CommandId, Dispatch, DispatchError, Request},
        metrics::Metric,
        state::KeyType,
        Hop,
    };

    #[test]
    fn test_stats_empty() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Stats, None);

        assert_eq!(
            Stats::dispatch(&hop, &req),
            Ok([
                ResponseType::Map as u8,
                // item len
                0,
                0,
            ]
            .to_vec()),
        );
    }

    #[test]
    fn test_stats_with_entry() {
        let hop = Hop::new();
        let req = Request::new(CommandId::Stats, None);

        hop.0.metrics_writer.increment(Metric::CommandsSuccessful);

        assert_eq!(
            Stats::dispatch(&hop, &req),
            Ok([
                ResponseType::Map as u8,
                // item count
                0,
                1,
                // item 1 key len
                19,
                // item 1 key
                b'c',
                b'o',
                b'm',
                b'm',
                b'a',
                b'n',
                b'd',
                b's',
                b'_',
                b's',
                b'u',
                b'c',
                b'c',
                b'e',
                b's',
                b's',
                b'f',
                b'u',
                b'l',
                // item 1 value len
                0,
                0,
                0,
                8,
                // item 1 value
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                1,
            ]
            .to_vec())
        )
    }

    #[test]
    fn test_stats_errors_with_key_type() {
        let hop = Hop::new();
        let req = Request::new_with_type(CommandId::Stats, None, KeyType::Map);

        assert!(matches!(
            Stats::dispatch(&hop, &req),
            Err(DispatchError::KeyTypeUnexpected)
        ));
    }
}
