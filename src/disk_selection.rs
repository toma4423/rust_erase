use std::process::Command; // Commandをインポート
use std::io::{self, BufRead};  // BufReadトレイトをインポート

/// ディスクを選択する関数
pub fn select_disk() -> Option<String> {
    let disks = get_available_disks(); // 実際のディスク情報を取得

    println!("------------------------------------");
    println!("Select Disk to Erase");
    println!("------------------------------------");

    for (i, disk) in disks.iter().enumerate() {
        println!("{}: {}", i + 1, disk);
    }

    println!("Enter the number of the disk you want to erase:");

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        println!("Error reading input.");
        return None;
    }

    match input.trim().parse::<usize>() {
        Ok(index) if index > 0 && index <= disks.len() => {
            Some(disks[index - 1].clone()) // 選択されたディスクを返す
        }
        _ => {
            println!("Invalid input. No disk selected.");
            None
        }
    }
}

/// 利用可能なディスク情報を取得する関数
fn get_available_disks() -> Vec<String> {
    // lsblk コマンドでディスクのデバイス名を取得
    let output = Command::new("lsblk")
        .arg("-d")
        .arg("-o")
        .arg("NAME,TRAN")
        .output()
        .expect("Failed to execute lsblk");

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut disks = vec![];

    // 取得したデバイス名に対してhdparmを実行して詳細情報を取得
    for line in output_str.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let disk_name = parts[0].trim();
            let transport = parts[1].trim();

            // hdparmコマンドを使用してディスク情報を取得
            if let Some((model, mut device_type)) = get_disk_info(disk_name) {
                if transport == "usb" {
                    // USB接続の場合はユーザーにHDDかSSDかを確認
                    device_type = prompt_user_for_device_type(&model, &mut io::stdin());
                }

                let disk_info = format!(
                    "/dev/{} - {} - {} - {}",
                    disk_name, model, device_type, transport.to_uppercase()
                );
                disks.push(disk_info);
            }
        }
    }

    disks
}

/// hdparmコマンドを使ってディスクのモデル名と回転速度（ディスクタイプ）を取得する関数
fn get_disk_info(disk_name: &str) -> Option<(String, String)> {
    // hdparm -I /dev/<disk_name> コマンドを実行
    let output = Command::new("sudo")
        .arg("hdparm")
        .arg("-I")
        .arg(format!("/dev/{}", disk_name))
        .output()
        .expect("Failed to execute hdparm");

    let output_str = String::from_utf8_lossy(&output.stdout);

    let mut model = "Unknown Model".to_string();
    let mut device_type = "Unknown".to_string();

    // hdparmの出力からモデル名と回転速度を取得
    for line in output_str.lines() {
        if line.contains("Model Number") {
            // モデル名の抽出
            if let Some(model_number) = line.split(":").nth(1) {
                model = model_number.trim().to_string();
            }
        }

        if line.contains("Rotation Rate") {
            // 回転速度からディスクタイプを判別
            if let Some(rotation_rate) = line.split(":").nth(1) {
                let rotation_rate = rotation_rate.trim();
                if rotation_rate == "Solid State Device" {
                    device_type = "SSD".to_string();
                } else if let Ok(rate) = rotation_rate.parse::<u32>() {
                    if rate > 0 {
                        device_type = "HDD".to_string();
                    }
                }
            }
        }
    }

    // モデル名が見つかった場合、モデル名とディスクタイプを返す
    if model != "Unknown Model" {
        Some((model, device_type))
    } else {
        None
    }
}

/// USB接続ディスクに対してユーザーにディスクタイプを尋ねる関数
fn prompt_user_for_device_type(model: &str, input: &mut dyn std::io::Read) -> String {
    println!(
        "The USB-connected disk '{}' was detected. Is this an HDD or SSD? (Enter 'h' for HDD or 's' for SSD):",
        model
    );

    let mut input_str = String::new();
    std::io::BufReader::new(input).read_line(&mut input_str).expect("Failed to read input");

    match input_str.trim() {
        "h" | "H" => "HDD".to_string(),
        "s" | "S" => "SSD".to_string(),
        _ => {
            println!("Invalid input. Defaulting to 'Unknown'.");
            "Unknown".to_string()
        }
    }
}
