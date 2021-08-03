use std::fs;

pub fn get_net_if_speed(device: &str) -> i32 {
    const DEFAULT_SPEED: i32 = 10000;

    let speed_path = format!("/sys/class/net/{}/speed", device);
    match fs::read_to_string(speed_path.clone()) {
        Ok(speed_str) => {
            return speed_str.parse::<i32>().unwrap_or(DEFAULT_SPEED);
        }
        Err(_) => {
            tracing::debug!(
                "Could not get speed from {}. Defaulting to 10 Gbps.",
                speed_path
            );
            DEFAULT_SPEED
        }
    }
}
