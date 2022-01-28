use std::time::{SystemTime, UNIX_EPOCH};

pub fn clock() -> f64 {
	    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as f64
}
