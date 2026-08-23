use chrono::Utc;

static mut INITIAL_TIME: Option<i64> = None;

fn get_now() -> i64 {
    Utc::now().timestamp_millis()
}

fn get_initial_time() -> i64 {
    unsafe {
        if let Some(x) = INITIAL_TIME {
            return x;
        }

        let now = get_now();
        INITIAL_TIME = Some(now);

        now
    }
}

pub fn timestamp_ms() -> i64 {
    get_now() - get_initial_time()
}
