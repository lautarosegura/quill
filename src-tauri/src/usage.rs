//! Monthly usage aggregation for the Uso page.
//! Reads directly from the HistoryStore's SQLite connection.

use chrono::{Datelike, NaiveDate, Utc};
use rusqlite::params;
use serde::Serialize;

use crate::engines::groq::catalog;
use crate::error::QuillError;
use crate::history::HistoryStore;

#[derive(Debug, Serialize)]
pub struct UsageStats {
    /// "2026-04" format.
    pub month: String,
    pub total_transcriptions: u64,
    pub local_transcriptions: u64,
    pub groq_transcriptions: u64,
    pub total_audio_seconds: u64,
    pub groq_audio_seconds: u64,
    pub estimated_groq_cost_usd: f64,
    /// Last 30 days including today, oldest first. Days with no entries
    /// have count 0 so the chart is contiguous.
    pub daily_counts: Vec<DailyCount>,
}

#[derive(Debug, Serialize)]
pub struct DailyCount {
    pub date: String, // YYYY-MM-DD
    pub count: u64,
}

/// Returns ISO-8601 string for first instant of `year-month-01` UTC and
/// first instant of the month after.
fn month_bounds(year: i32, month: u32) -> (String, String) {
    let start = NaiveDate::from_ymd_opt(year, month, 1)
        .expect("valid YMD")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let end = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("valid YMD")
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    (start.to_rfc3339(), end.to_rfc3339())
}

pub fn compute_for_current_month(store: &HistoryStore) -> Result<UsageStats, QuillError> {
    let now = Utc::now();
    let year = now.year();
    let month = now.month();
    let (start, end) = month_bounds(year, month);

    let conn_arc = store.connection();
    let conn = conn_arc.lock().unwrap();

    // Totals for current month, successes only.
    let (total, local_count, groq_count, total_ms, groq_ms): (u64, u64, u64, i64, i64) = conn
        .query_row(
            "SELECT
                COUNT(*),
                SUM(CASE WHEN engine = 'local' THEN 1 ELSE 0 END),
                SUM(CASE WHEN engine = 'groq'  THEN 1 ELSE 0 END),
                COALESCE(SUM(duration_ms), 0),
                COALESCE(SUM(CASE WHEN engine = 'groq' THEN duration_ms ELSE 0 END), 0)
             FROM transcriptions
             WHERE status = 'success'
               AND created_at >= ?1
               AND created_at <  ?2",
            params![start, end],
            |row| {
                let total: u64 = row.get::<_, Option<i64>>(0)?.unwrap_or(0) as u64;
                let local_count: u64 = row.get::<_, Option<i64>>(1)?.unwrap_or(0) as u64;
                let groq_count: u64 = row.get::<_, Option<i64>>(2)?.unwrap_or(0) as u64;
                let total_ms: i64 = row.get::<_, Option<i64>>(3)?.unwrap_or(0);
                let groq_ms: i64 = row.get::<_, Option<i64>>(4)?.unwrap_or(0);
                Ok((total, local_count, groq_count, total_ms, groq_ms))
            },
        )
        .map_err(|e| QuillError::Other(format!("usage totals: {e}")))?;

    // Per-model Groq milliseconds, so we can bill each chunk at its own rate.
    let mut per_model_groq_ms: Vec<(String, i64)> = Vec::new();
    {
        let mut stmt = conn
            .prepare(
                "SELECT COALESCE(model, ''), COALESCE(SUM(duration_ms), 0)
                 FROM transcriptions
                 WHERE status = 'success'
                   AND engine = 'groq'
                   AND created_at >= ?1
                   AND created_at <  ?2
                 GROUP BY model",
            )
            .map_err(|e| QuillError::Other(format!("usage per-model prepare: {e}")))?;
        let rows = stmt
            .query_map(params![start, end], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            })
            .map_err(|e| QuillError::Other(format!("usage per-model query: {e}")))?;
        for r in rows {
            per_model_groq_ms.push(r.map_err(|e| QuillError::Other(e.to_string()))?);
        }
    }

    let estimated_groq_cost_usd: f64 = per_model_groq_ms
        .iter()
        .map(|(model, ms)| {
            let hours = (*ms as f64) / (1000.0 * 60.0 * 60.0);
            let rate = catalog::cost_per_hour_for(model).unwrap_or(0.04);
            hours * rate
        })
        .sum();

    // Daily counts for last 30 days.
    let thirty_days_ago = (now - chrono::Duration::days(29))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    let daily_start = thirty_days_ago.to_rfc3339();

    let mut day_counts_by_date: std::collections::HashMap<String, u64> = Default::default();
    {
        let mut stmt = conn
            .prepare(
                "SELECT substr(created_at, 1, 10) AS day, COUNT(*)
                 FROM transcriptions
                 WHERE status = 'success'
                   AND created_at >= ?1
                 GROUP BY day",
            )
            .map_err(|e| QuillError::Other(format!("usage daily prepare: {e}")))?;
        let rows = stmt
            .query_map(params![daily_start], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as u64))
            })
            .map_err(|e| QuillError::Other(format!("usage daily query: {e}")))?;
        for r in rows {
            let (day, count) = r.map_err(|e| QuillError::Other(e.to_string()))?;
            day_counts_by_date.insert(day, count);
        }
    }

    let mut daily_counts = Vec::with_capacity(30);
    for i in 0..30 {
        let date = (thirty_days_ago + chrono::Duration::days(i))
            .date_naive()
            .format("%Y-%m-%d")
            .to_string();
        let count = day_counts_by_date.get(&date).copied().unwrap_or(0);
        daily_counts.push(DailyCount { date, count });
    }

    Ok(UsageStats {
        month: format!("{:04}-{:02}", year, month),
        total_transcriptions: total,
        local_transcriptions: local_count,
        groq_transcriptions: groq_count,
        total_audio_seconds: (total_ms / 1000).max(0) as u64,
        groq_audio_seconds: (groq_ms / 1000).max(0) as u64,
        estimated_groq_cost_usd,
        daily_counts,
    })
}
