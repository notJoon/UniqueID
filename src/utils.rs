use std::{
    hint::spin_loop,
    time::{Duration, SystemTime},
};

pub fn get_timestamp(epoch: SystemTime) -> i64 {
    SystemTime::now()
        .duration_since(epoch)
        .unwrap_or(Duration::default())
        .as_millis() as i64
}

pub fn get_epoch() -> SystemTime {
    SystemTime::UNIX_EPOCH
}

pub fn bind_time(timestamp: i64, epoch: SystemTime) -> i64 {
    let mut very_last_time: i64;

    loop {
        very_last_time = get_timestamp(epoch);
        if very_last_time > timestamp {
            return very_last_time;
        }

        spin_loop();
    }
}
