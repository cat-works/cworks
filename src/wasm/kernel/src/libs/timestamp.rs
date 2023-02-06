use chrono::Utc;

pub fn timestamp() -> f32 {
    (Utc::now().timestamp_millis() - 1672498800000) as f32 / 1000.0
}
