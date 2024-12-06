use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_message(action: &str, result: &str, details: &str) {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    let timestamp = format!("{:?}", since_the_epoch);

    let log_entry = format!(
        "[{}] ACTION: {}\nRESULT: {}\nDETAILS: {}\n",
        timestamp, action, result, details
    );

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("erasure_log.txt")
        .expect("Failed to open log file.");
    writeln!(file, "{}", log_entry).expect("Failed to write to log file.");
}
