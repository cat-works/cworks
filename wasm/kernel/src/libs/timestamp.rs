use std::{sync::OnceLock, time::Instant};

static INITIAL_TIME: OnceLock<Instant> = OnceLock::new();

pub fn timestamp_ms() -> i64 {
    INITIAL_TIME
        .get_or_init(Instant::now)
        .elapsed()
        .as_millis() as i64
}
