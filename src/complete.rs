use std::io::{self};

pub fn show() {
    println!("Disk erase completed successfully.");
    println!("------------------------------------");
    println!("Press Enter to shut down the system or Q + Enter to quit the program.");

    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        eprintln!("Failed to read input: {}", e);
        return;
    }

    match input.trim().to_lowercase().as_str() {
        "" => {
            println!("Shutting down the system...");
            // システムシャットダウン処理を実行
            if let Err(e) = std::process::Command::new("sudo").arg("shutdown").arg("now").status() {
                eprintln!("Failed to shut down the system: {}", e);
            }
        }
        "q" => {
            println!("Program exited without shutting down.");
            // プログラムの終了
        }
        _ => {
            println!("Invalid input. Program exited without shutting down.");
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_show() {
        // テストは実行可能だが、ユーザー入力が必要なため模擬的なテスト
    }
}
