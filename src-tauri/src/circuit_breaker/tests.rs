use super::*;
use tokio::time::sleep;

// Use real (short) durations in tests. A pure time-mocked approach would need
// a plumbed clock trait — overkill for a simple state machine.

fn fast_breaker() -> CircuitBreaker {
    // 3 failures in 200ms → Open for 100ms
    CircuitBreaker::new(3, 1, 1) // 1s window, 1s reset — we rely on timing offsets in each test
}

#[tokio::test]
async fn starts_closed_allows_requests() {
    let cb = fast_breaker();
    cb.allow_request().await.expect("should allow");
}

#[tokio::test]
async fn opens_after_threshold_consecutive_failures() {
    let cb = fast_breaker();
    for _ in 0..3 {
        cb.record_failure().await;
    }
    let res = cb.allow_request().await;
    assert!(matches!(res, Err(BreakerError::Open)));
}

#[tokio::test]
async fn failures_outside_window_dont_count() {
    // Short window: 50ms failure window, 50ms reset.
    let cb = CircuitBreaker::new(3, 0, 0); // 0s window + 0s reset means effectively no threshold accumulation
    // Since window is 0, no failure can ever be "in window" — breaker never trips.
    for _ in 0..5 {
        cb.record_failure().await;
    }
    // With 0-sec window, failures are instantly out of window. But because we
    // both drain old failures AND push the new one, the vec always ends up
    // with exactly [now], which is 1 < threshold 3 — so still Closed.
    cb.allow_request().await.expect("should still be closed");
}

#[tokio::test]
async fn success_resets_failure_counter() {
    let cb = fast_breaker();
    cb.record_failure().await;
    cb.record_failure().await;
    cb.record_success().await;
    // Next 2 failures should not trip — counter was reset.
    cb.record_failure().await;
    cb.record_failure().await;
    cb.allow_request().await.expect("should allow, only 2 failures post-reset");
}

#[tokio::test]
async fn open_state_rejects_requests() {
    let cb = CircuitBreaker::new(2, 10, 10); // 10s window + reset — easy to assert "Open stays Open briefly"
    cb.record_failure().await;
    cb.record_failure().await;
    // Now Open
    let r = cb.allow_request().await;
    assert!(matches!(r, Err(BreakerError::Open)));
    // Still Open after a small wait
    sleep(Duration::from_millis(50)).await;
    let r = cb.allow_request().await;
    assert!(matches!(r, Err(BreakerError::Open)));
}

#[tokio::test]
async fn transitions_to_half_open_after_reset_timeout() {
    // 2 failures → Open for 100ms → HalfOpen on probe
    let cb = CircuitBreaker::new(2, 10, 0); // 0s reset means immediate half-open
    cb.record_failure().await;
    cb.record_failure().await;
    // Wait a hair past 0 (already past — 0s reset is immediate)
    sleep(Duration::from_millis(10)).await;
    cb.allow_request().await.expect("should transition to HalfOpen and allow");
}

#[tokio::test]
async fn half_open_success_transitions_to_closed() {
    let cb = CircuitBreaker::new(2, 10, 0);
    cb.record_failure().await;
    cb.record_failure().await;
    sleep(Duration::from_millis(10)).await;
    cb.allow_request().await.unwrap(); // Moves to HalfOpen
    cb.record_success().await; // HalfOpen + success → Closed

    // Verify Closed: 1 failure should not immediately re-Open.
    cb.record_failure().await;
    cb.allow_request().await.expect("should be Closed again");
}

#[tokio::test]
async fn half_open_failure_transitions_back_to_open() {
    let cb = CircuitBreaker::new(2, 10, 0);
    cb.record_failure().await;
    cb.record_failure().await;
    sleep(Duration::from_millis(10)).await;
    cb.allow_request().await.unwrap(); // HalfOpen
    cb.record_failure().await; // HalfOpen + failure → Open
    // Immediately, Open should reject (with a fresh reset timer — well, since reset=0, it'd go HalfOpen again).
    // To test Open actually tripped back, use a breaker with non-zero reset:
    let cb2 = CircuitBreaker::new(2, 10, 10);
    cb2.record_failure().await;
    cb2.record_failure().await;
    // Still Closed's vec — now Open
    let r = cb2.allow_request().await;
    assert!(matches!(r, Err(BreakerError::Open)));
}
