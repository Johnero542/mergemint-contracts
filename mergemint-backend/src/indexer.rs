//! Minimal indexer coordination helpers.
//!
//! These helpers keep defensive polling behavior local and testable:
//! - debounce fire-and-forget full polls after submissions so bursts do not
//!   duplicate targeted refresh work;
//! - avoid advancing the stored ledger on suspicious empty event responses.

use std::time::{Duration, Instant};

/// Tracks the last full indexer poll triggered after a transaction submission.
#[derive(Debug, Clone)]
pub struct PollDebouncer {
    last_full_poll: Option<Instant>,
    min_interval: Duration,
}

impl PollDebouncer {
    pub fn new(min_interval: Duration) -> Self {
        Self {
            last_full_poll: None,
            min_interval,
        }
    }

    /// Returns true when a full poll should be triggered now.
    pub fn should_poll_now(&mut self, now: Instant) -> bool {
        let should_poll = self
            .last_full_poll
            .map(|last| now.duration_since(last) >= self.min_interval)
            .unwrap_or(true);

        if should_poll {
            self.last_full_poll = Some(now);
        }

        should_poll
    }
}

/// Decision returned after an indexer poll response is evaluated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerAdvance {
    AdvanceTo(u64),
    KeepCurrent,
}

/// Decide whether it is safe to advance the persisted last-ledger cursor.
///
/// Empty event pages are only trusted when the latest ledger is at or before
/// the requested range end. If the node reports a later latest ledger while
/// returning no events for the range, we keep the current cursor and warn so a
/// subsequent poll retries the same boundary.
pub fn ledger_advance_after_poll(
    current_last_ledger: u64,
    requested_to_ledger: u64,
    latest_ledger: u64,
    event_count: usize,
) -> LedgerAdvance {
    if event_count == 0 && latest_ledger > requested_to_ledger {
        tracing::warn!(
            current_last_ledger,
            requested_to_ledger,
            latest_ledger,
            "empty get_events response looks suspicious; keeping indexer cursor"
        );
        return LedgerAdvance::KeepCurrent;
    }

    LedgerAdvance::AdvanceTo(requested_to_ledger)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debouncer_skips_full_poll_inside_interval() {
        let start = Instant::now();
        let mut debouncer = PollDebouncer::new(Duration::from_secs(10));

        assert!(debouncer.should_poll_now(start));
        assert!(!debouncer.should_poll_now(start + Duration::from_secs(1)));
        assert!(debouncer.should_poll_now(start + Duration::from_secs(10)));
    }

    #[test]
    fn empty_response_with_later_latest_ledger_keeps_cursor() {
        assert_eq!(
            ledger_advance_after_poll(100, 120, 121, 0),
            LedgerAdvance::KeepCurrent
        );
    }

    #[test]
    fn empty_response_at_latest_ledger_advances_cursor() {
        assert_eq!(
            ledger_advance_after_poll(100, 120, 120, 0),
            LedgerAdvance::AdvanceTo(120)
        );
    }

    #[test]
    fn non_empty_response_advances_cursor() {
        assert_eq!(
            ledger_advance_after_poll(100, 120, 130, 1),
            LedgerAdvance::AdvanceTo(120)
        );
    }
}
