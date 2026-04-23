//! Monthly cost-alert logic for Groq usage.
//!
//! Call [`check`] after every successful Groq transcription; when the user's
//! estimated monthly cost crosses the configured threshold, it returns an
//! [`Alert`] exactly once per month. The "already fired this month" bit is
//! persisted at `~/.quill/alert_state.json`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::QuillError;
use crate::history::HistoryStore;
use crate::paths;
use crate::usage;

#[derive(Debug, Default, Serialize, Deserialize)]
struct AlertState {
    /// ISO year-month, e.g. "2026-04". `None` means never fired.
    last_fired_month: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub month: String,
    pub current_cost_usd: f64,
    pub threshold_usd: f64,
}

fn state_file() -> PathBuf {
    paths::quill_home().join("alert_state.json")
}

fn load_state() -> AlertState {
    std::fs::read_to_string(state_file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &AlertState) {
    if let Ok(json) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(state_file(), json);
    }
}

/// Pure decision: given cost stats, a threshold, and what month we last
/// alerted for, should we fire an alert this time? Split out from [`check`]
/// so the logic can be unit-tested without touching disk or the DB.
fn decide(
    month: &str,
    current_cost_usd: f64,
    threshold_usd: f64,
    last_fired_month: Option<&str>,
) -> Option<Alert> {
    if threshold_usd <= 0.0 {
        return None;
    }
    if current_cost_usd < threshold_usd {
        return None;
    }
    if last_fired_month == Some(month) {
        return None;
    }
    Some(Alert {
        month: month.to_string(),
        current_cost_usd,
        threshold_usd,
    })
}

/// Returns `Some(Alert)` iff the user's current month Groq cost is >= threshold
/// AND we have not already fired an alert for this month. The "fired" bit is
/// recorded as a side effect before returning, so callers do not need to track it.
pub fn check(history: &HistoryStore, threshold_usd: f64) -> Result<Option<Alert>, QuillError> {
    let stats = usage::compute_for_current_month(history)?;
    let state = load_state();
    let Some(alert) = decide(
        &stats.month,
        stats.estimated_groq_cost_usd,
        threshold_usd,
        state.last_fired_month.as_deref(),
    ) else {
        return Ok(None);
    };
    save_state(&AlertState {
        last_fired_month: Some(stats.month.clone()),
    });
    Ok(Some(alert))
}

/// Clears the "fired this month" bit. Exposed for the Settings page so the
/// user can re-arm the alert after dismissing one.
pub fn reset() -> Result<(), QuillError> {
    save_state(&AlertState::default());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fires_when_cost_crosses_threshold_and_not_yet_alerted() {
        let alert = decide("2026-04", 12.0, 10.0, None).expect("should fire");
        assert_eq!(alert.month, "2026-04");
        assert_eq!(alert.current_cost_usd, 12.0);
        assert_eq!(alert.threshold_usd, 10.0);
    }

    #[test]
    fn does_not_fire_when_below_threshold() {
        assert!(decide("2026-04", 9.99, 10.0, None).is_none());
    }

    #[test]
    fn does_not_fire_when_already_alerted_this_month() {
        assert!(decide("2026-04", 50.0, 10.0, Some("2026-04")).is_none());
    }

    #[test]
    fn fires_again_in_a_new_month() {
        // Previous alert was March; April cost crosses threshold.
        let alert = decide("2026-04", 12.0, 10.0, Some("2026-03")).expect("new month should fire");
        assert_eq!(alert.month, "2026-04");
    }

    #[test]
    fn zero_or_negative_threshold_never_fires() {
        assert!(decide("2026-04", 50.0, 0.0, None).is_none());
        assert!(decide("2026-04", 50.0, -1.0, None).is_none());
    }

    #[test]
    fn fires_exactly_at_threshold() {
        // Boundary: cost == threshold is treated as crossed.
        assert!(decide("2026-04", 10.0, 10.0, None).is_some());
    }
}
