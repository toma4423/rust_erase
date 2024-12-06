use crate::hdd_erase;
use crate::ssd_erase;
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

/// ログファイルへの出力
fn log_message(action: &str, result: &str, details: &str) {
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

/// 消去プロセスのエントリーポイント
pub fn start(selected_disks: &Vec<String>) {
    println!("------------------------------------");
    println!("Erasing Disks...");
    println!("------------------------------------");

    let errors: Vec<String> = selected_disks
        .par_iter()
        .map(|disk| {
            println!("Erasing {}...", disk);
            log_message(
                &format!("Starting erase for {}", disk),
                "In Progress",
                "Initiating disk erasure process.",
            );

            let result = if disk.contains("HDD") {
                hdd_erase::erase_hdd_with_dod5220(disk)
            } else if disk.contains("SSD") {
                ssd_erase::secure_erase_ssd(disk)
            } else {
                Err(format!("Unknown disk type for {}. Skipping...", disk))
            };

            match result {
                Ok(_) => {
                    log_message(
                        &format!("Erase complete for {}", disk),
                        "Success",
                        "Disk erased successfully.",
                    );
                    String::new() // エラーなし
                }
                Err(e) => {
                    log_message(&format!("Erase failed for {}", disk), "Error", &e);
                    e.to_string() // String型に変換
                }
            }
        })
        .filter(|error| !error.is_empty())
        .collect(); // エラーのあるものだけをフィルタリング

    println!("Erasure complete.");
    log_message(
        "Erasure process",
        "Complete",
        "Disk erasure process completed for all selected disks.",
    );

    if !errors.is_empty() {
        log_message(
            "Errors encountered during erasure",
            "Error",
            &format!("{:?}", errors),
        );
    } else {
        log_message(
            "Erasure Summary",
            "No errors",
            "All disks erased successfully.",
        );
    }
}
