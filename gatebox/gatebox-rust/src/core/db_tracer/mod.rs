// From app/modules/core/db_tracer/db_tracer.go
// Counts DB operations and total time (microseconds). No sqlx tracer hook; use manually or wire later.

use std::sync::atomic::{AtomicU64, Ordering};

/// Counts queries and total time spent in DB (microseconds).
#[derive(Debug, Default)]
pub struct DBTracer {
    query_count: AtomicU64,
    total_time_us: AtomicU64,
}

impl DBTracer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Call at query start (e.g. from a wrapper). Returns start instant for computing elapsed.
    #[inline]
    pub fn trace_query_start(&self) -> std::time::Instant {
        std::time::Instant::now()
    }

    /// Call at query end with the instant from trace_query_start.
    pub fn trace_query_end(&self, start: std::time::Instant) {
        self.query_count.fetch_add(1, Ordering::Relaxed);
        let us = start.elapsed().as_micros() as u64;
        self.total_time_us.fetch_add(us, Ordering::Relaxed);
    }

    /// Returns (count, total_ms, avg_ms).
    pub fn get_stats(&self) -> (u64, u64, f64) {
        let c = self.query_count.load(Ordering::Relaxed);
        let us = self.total_time_us.load(Ordering::Relaxed);
        let total_ms = us / 1000;
        let avg_ms = if c > 0 {
            (us as f64) / (c as f64) / 1000.0
        } else {
            0.0
        };
        (c, total_ms, avg_ms)
    }

    pub fn reset(&self) {
        self.query_count.store(0, Ordering::Relaxed);
        self.total_time_us.store(0, Ordering::Relaxed);
    }
}

/// Global tracer (shared by read/write pools when wired).
pub static GLOBAL: std::sync::OnceLock<DBTracer> = std::sync::OnceLock::new();

/// Returns the global DBTracer instance.
pub fn global() -> &'static DBTracer {
    GLOBAL.get_or_init(DBTracer::new)
}
