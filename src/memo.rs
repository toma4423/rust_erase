use crossterm::{
    ExecutableCommand, cursor, terminal, event,
    event::{KeyCode, KeyEvent, Event},
};
use std::io::{stdout, Write};
use std::process::Command;
use std::time::{Duration, Instant};

fn main() {
    // シミュレートされたディスク情報
    let disks = vec![
        "Disk1 - 500GB - SSD - SATA",
        "Disk2 - 1000GB - HDD - SATA",
        "Disk3 - 250GB - SSD - NVMe",
    ];

    // ディスク選択画面を表示して、ユーザーが選択したディスクを取得
    let selected_disks = select_disks(disks);

    if selected_disks.is_empty() {
        println!("No disks selected. Exiting program.");
        return;
    }

    // 確認画面
    if !confirm_disk_erasure(&selected_disks) {
        println!("Operation canceled by the user.");
        return;
    }

    // ディスクの消去を開始
    for disk in selected_disks {
        if disk.contains("HDD") {
            erase_hdd_with_dod5220(&disk);
        } else if disk.contains("SSD") && disk.contains("SATA") {
            secure_erase_sata_ssd(&disk);
        } else if disk.contains("NVMe") {
            secure_erase_nvme_ssd(&disk);
        }
    }

    // シャットダウン処理
    shutdown_system();
}

/// ユーザーにディスクを選択させる関数
fn select_disks(disks: Vec<&str>) -> Vec<String> {
    let mut stdout = stdout();
    let mut selected_disk_indices = vec![];
    let mut current_selection = 0;

    loop {
        // 画面をクリア
        stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();

        // ディスクの一覧を表示
        for (i, disk) in disks.iter().enumerate() {
            if i == current_selection {
                stdout.write_all(format!("> {}\n", disk).as_bytes()).unwrap();
            } else {
                stdout.write_all(format!("  {}\n", disk).as_bytes()).unwrap();
            }
        }
        stdout.flush().unwrap();

        // キー入力待ち
        if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
            match code {
                KeyCode::Up => {
                    if current_selection > 0 {
                        current_selection -= 1;
                    }
                }
                KeyCode::Down => {
                    if current_selection < disks.len() - 1 {
                        current_selection += 1;
                    }
                }
                KeyCode::Enter => {
                    // ディスクの選択/解除
                    if selected_disk_indices.contains(&current_selection) {
                        selected_disk_indices.retain(|&x| x != current_selection);
                    } else {
                        selected_disk_indices.push(current_selection);
                    }
                }
                KeyCode::Esc => {
                    // プログラム終了
                    stdout.execute(terminal::Clear(terminal::ClearType::All)).unwrap();
                    println!("Canceled by user.");
                    std::process::exit(0);
                }
                KeyCode::Char('q') => {
                    // 確定して終了
                    return selected_disk_indices
                        .iter()
                        .map(|&i| disks[i].to_string())
                        .collect();
                }
                _ => {}
            }
        }
    }
}

/// 確認画面
fn confirm_disk_erasure(selected_disks: &Vec<String>) -> bool {
    println!("The following disks will be erased:");
    for disk in selected_disks {
        println!("{}", disk);
    }
    println!("Are you sure you want to proceed? (Y/N)");

    loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
            match code {
                KeyCode::Char('y') | KeyCode::Char('Y') => return true,
                KeyCode::Char('n') | KeyCode::Char('N') => return false,
                _ => {}
            }
        }
    }
}

/// SATA-HDD用のDoD5220.22-M方式消去
fn erase_hdd_with_dod5220(device: &str) {
    println!("Starting DoD5220.22-M wipe on: {}", device);
    for _ in 0..3 {
        Command::new("sh")
            .arg("-c")
            .arg(format!("dd if=/dev/urandom of={} bs=4M", device))
            .output()
            .expect("Failed to overwrite with random data");
    }
    Command::new("sh")
        .arg("-c")
        .arg(format!("dd if=/dev/zero of={} bs=4M", device))
        .output()
        .expect("Failed to overwrite with zeros");
    println!("DoD5220.22-M wipe completed for: {}", device);
}

/// SATA-SSD用のATA Secure Erase
fn secure_erase_sata_ssd(device: &str) {
    println!("Starting ATA Secure Erase on: {}", device);
    Command::new("sudo")
        .arg("hdparm")
        .arg("--user-master")
        .arg("u")
        .arg("--security-erase")
        .arg("enhanced")
        .arg(device)
        .output()
        .expect("Failed to perform ATA Secure Erase");
    println!("ATA Secure Erase completed for: {}", device);
}

/// NVMe-SSD用のSecure Erase
fn secure_erase_nvme_ssd(device: &str) {
    println!("Starting NVMe Secure Erase on: {}", device);
    Command::new("sudo")
        .arg("nvme")
        .arg("format")
        .arg(device)
        .arg("--ses=1")
        .output()
        .expect("Failed to perform NVMe Secure Erase");
    println!("NVMe Secure Erase completed for: {}", device);
}

/// シャットダウン処理
fn shutdown_system() {
    println!("Shutting down system...");
    Command::new("sudo")
        .arg("shutdown")
        .arg("now")
        .output()
        .expect("Failed to shutdown the system");
}
