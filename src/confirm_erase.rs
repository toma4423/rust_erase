pub fn confirm(selected_disks: &Vec<String>) -> bool {
    println!("------------------------------------");
    println!("Confirm Erasure");
    println!("------------------------------------");
    println!("Are you sure you want to erase the selected disks?");
    println!("Disks to be erased:");
    for disk in selected_disks {
        println!("- {}", disk);
    }
    println!("Enter: Start Erase | q: Cancel");

    let mut input = String::new();
    if std::io::stdin().read_line(&mut input).is_err() {
        println!("Error reading input. Erase cancelled.");
        return false;
    }

    match input.trim() {
        "" => {
            println!("Erase process started..."); // Enterが押されたとき
            true // 消去プロセスを続行
        }
        "q" => {
            println!("Erase cancelled. Exiting..."); // qが押されたとき
            false // プロセスを中止
        }
        _ => {
            println!("Invalid input. Erase cancelled. Exiting..."); // その他のキーが押された場合
            false
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_confirm_continue() {
        // 模擬的なテストケース (標準入力が関わるため、実際のテストでは難しい)
    }
}
