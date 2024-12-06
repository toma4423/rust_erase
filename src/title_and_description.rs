pub fn show() {
    println!("----------------------------------------------");
    println!("  disk_eraser_(~~Angel of Destruction, Forko~~)");
    println!("----------------------------------------------");

    println!("  Welcome to the Disk Eraser Utility.");
    println!("");
    println!("  This program is designed to securely erase all data from your selected disk(s) using the DoD 5220.22-M standard.");
    println!("  The following operations will be performed on the selected disk:");
    println!("");
    println!("  - Multiple passes of random data and zeros will be written.");
    println!("  - The final pass will overwrite the disk with zeros.");
    println!("  - Once the process starts, it cannot be interrupted.");
    println!("");
    println!("  WARNING:");
    println!("  - This program will irreversibly destroy all data on the selected disk(s).");
    println!("  - Ensure that you have backed up any important data before proceeding.");
    println!("  - The program is designed to work with HDDs and SSDs, but make sure you have selected the correct disk before starting.");
    println!("");
    println!("  Disclaimer:");
    println!("  The author of this program is not responsible for any data loss or damage caused by the use of this tool.");
    println!("");
    println!("----------------------------------------------");
    println!("  Press Enter to continue and select a disk to erase...");

    // 入力を受け取る処理を関数内に移動
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}
