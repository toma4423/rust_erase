use crate::logger::log_message;
use std::process::Command;

pub fn secure_erase_ssd(device: &str) -> Result<(), String> {
    let device_name = device.split_whitespace().next().unwrap_or(device);

    println!("Starting Secure Erase on: {}", device_name);
    log_message(
        &format!("Secure Erase started on: {}", device_name),
        "In Progress",
        "Secure Erase will be performed based on device type.",
    );

    if device.contains("SATA") {
        // パスワードを設定（ATA Secure Erase）
        let output = Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-set-pass")
            .arg("0000")
            .arg(device_name)
            .output()
            .map_err(|e| format!("Failed to set password: {}", e))?;

        if !output.status.success() {
            let error_message = format!(
                "Failed to set password for {}: {}",
                device_name,
                String::from_utf8_lossy(&output.stderr)
            );
            log_message("ATA Secure Erase", "Failed", &error_message);
            return Err(error_message);
        }

        // Enhanced Secure Eraseを試みる
        let output = Command::new("sudo")
            .arg("hdparm")
            .arg("--user-master")
            .arg("u")
            .arg("--security-erase-enhanced")
            .arg("0000")
            .arg(device_name)
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    log_message(
                        "ATA Secure Erase",
                        "Success",
                        &format!("Enhanced Secure Erase completed for {}", device_name),
                    );
                } else {
                    // 標準エラーメッセージを小文字に変換してエラーチェック
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    let stderr_lower = stderr.to_lowercase(); // 大文字小文字を区別しない

                    if stderr_lower.contains("feature not supported")
                        || stderr_lower.contains("invalid argument")
                    {
                        // Enhanced Eraseがサポートされていない場合、通常のセキュリティ消去を試行
                        log_message(
                            "ATA Secure Erase",
                            "Info",
                            "Enhanced Erase not supported, falling back to standard erase.",
                        );
                        let output = Command::new("sudo")
                            .arg("hdparm")
                            .arg("--user-master")
                            .arg("u")
                            .arg("--security-erase")
                            .arg("0000")
                            .arg(device_name)
                            .output()
                            .map_err(|e| {
                                format!("Failed to start standard ATA Secure Erase: {}", e)
                            })?;

                        if output.status.success() {
                            log_message(
                                "ATA Secure Erase",
                                "Success",
                                &format!("Standard Secure Erase completed for {}", device_name),
                            );
                        } else {
                            let error_message = String::from_utf8_lossy(&output.stderr);
                            log_message("ATA Secure Erase", "Failed", &error_message);
                            return Err(error_message.to_string());
                        }
                    } else {
                        let error_message = format!(
                            "ATA Enhanced Secure Erase failed for {}: {}",
                            device_name, stderr
                        );
                        log_message("ATA Secure Erase", "Failed", &error_message);
                        return Err(error_message);
                    }
                }
            }
            Err(e) => {
                let error_message = format!("Failed to start Enhanced Secure Erase: {}", e);
                log_message("ATA Secure Erase", "Failed", &error_message);
                return Err(error_message);
            }
        }
    }

    Ok(())
}
