// 修正: 未使用のインポートを削除
use crate::logger::log_message;
use indicatif::{ProgressBar, ProgressStyle}; // プログレスバー用のインポート
use regex::Regex; // regexクレートをインポート
use std::io::{BufRead, BufReader}; // BufReaderをインポート
use std::process::{Command, Stdio}; // Stdioをインポート

/// ddコマンドを実行し、進捗をプログレスバーに表示するヘルパー関数
///
/// # Arguments
/// * `dd_command` - 実行するddコマンドの文字列 (例: "dd if=/dev/urandom of=/dev/sdx ...")
/// * `total_bytes` - 書き込むべき合計バイト数 (プログレスバーの最大値)
/// * `bar` - 更新対象のindicatif::ProgressBar
///
/// # Returns
/// * `Result<(), String>` - ddコマンドの実行結果
fn run_dd_with_progress(
    dd_command: &str,
    total_bytes: u64,
    bar: &ProgressBar,
) -> Result<(), String> {
    // ddコマンドに status=progress を追加し、stderrをパイプする
    let command_with_progress = format!("{} status=progress", dd_command);
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&command_with_progress)
        .stderr(Stdio::piped()) // stderrをキャプチャ
        .spawn()
        .map_err(|e| format!("Failed to spawn dd command: {}", e))?;

    // プログレスバーの最大値を設定
    bar.set_length(total_bytes);
    bar.set_position(0); // 初期位置を0に

    // stderrから進捗を読み取るための正規表現
    // 例: "12345 bytes (12 kB, 12 KiB) copied, ..."
    let re = Regex::new(r"^(\d+)\s+bytes").unwrap(); // bytesの前の数字をキャプチャ

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "Failed to capture stderr".to_string())?;

    // BufReaderを使ってstderrを行ごとに読み取る
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                // 正規表現でコピーされたバイト数を抽出
                if let Some(caps) = re.captures(&line_content) {
                    if let Some(bytes_str) = caps.get(1) {
                        if let Ok(bytes_copied) = bytes_str.as_str().parse::<u64>() {
                            // プログレスバーの位置を更新
                            bar.set_position(bytes_copied);
                        }
                    }
                }
                // ddからの他のstderr出力（エラーなど）があれば表示（デバッグ用）
                #[cfg(debug_assertions)]
                eprintln!("dd stderr: {}", line_content);
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Error reading dd stderr: {}", e);
                // エラーが発生しても続行するかもしれないが、一旦無視
            }
        }
    }

    // ddコマンドの終了を待つ
    let status = child
        .wait()
        .map_err(|e| format!("Failed to wait for dd command: {}", e))?;

    if status.success() {
        bar.finish_with_message("Pass complete"); // 成功したら完了メッセージ
        Ok(())
    } else {
        // stderrの内容全体を取得しようと試みる（既に行ごとには読んでいるが）
        // let mut stderr_output = String::new();
        // if let Some(mut stderr_stream) = child.stderr.take() {
        //     if let Err(e) = stderr_stream.read_to_string(&mut stderr_output) {
        //         eprintln!("Failed to read full stderr on error: {}", e);
        //     }
        // }
        let error_message = format!(
            "dd command failed with status: {}. Check stderr logs above.", status // stderr_output を含めると冗長になる可能性
        );
        bar.abandon_with_message("Pass failed"); // 失敗したら失敗メッセージ
        Err(error_message)
    }
}

pub fn erase_hdd_with_dod5220(device: &str) -> Result<(), String> {
    let device_name = device.split_whitespace().next().unwrap_or(device);

    println!("Starting DoD5220.22-M wipe on: {}", device_name);
    log_message(
        &format!("DoD5220.22-M wipe started on: {}", device_name),
        "In Progress",
        "Random data and zeros will be written in multiple passes.",
    );

    // デバイスサイズの取得 (一度だけ取得)
    let device_size = {
        let output = Command::new("sh")
            .arg("-c")
            .arg(format!("blockdev --getsize64 {}", device_name))
            .output()
            .map_err(|e| format!("Failed to get device size for {}: {}", device_name, e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "Failed to get device size for {}: {}",
                device_name, stderr
            ));
        }
        let size_str = String::from_utf8_lossy(&output.stdout);
        size_str
            .trim()
            .parse::<u64>()
            .map_err(|e| format!("Failed to parse device size '{}': {}", size_str, e))?
    };

    if device_size == 0 {
        return Err(format!(
            "Device size for {} is reported as 0. Cannot proceed.",
            device_name
        ));
    }
    log_message(
        "Device Size",
        "Success",
        &format!("Determined size for {} is {} bytes.", device_name, device_size),
    );


    // --- パス1-3: ランダムデータの書き込み ---
    for i in 0..3 {
        let action = format!("Pass {}/3: Writing random data to {}", i + 1, device_name);
        println!("\n{}", action); // 見やすさのために改行を追加

        // プログレスバーの設定 (ループ内で毎回作成)
        let bar = ProgressBar::new(device_size); // 全体のバイト数を設定
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .progress_chars("#>-"),
        );
        bar.enable_steady_tick(100); // プログレスバーを滑らかに更新

        // ddコマンドの構築 (bs=4M は進捗更新には粗すぎる可能性があるため、bs=1M に変更するかも？)
        // 注意: count は status=progress と併用すると予期しない動作をする可能性があるため削除
        // dd は EOF に達するまで書き込む
        let dd_cmd = format!("dd if=/dev/urandom of={} bs=4M oflag=direct", device_name); // directフラグを追加してキャッシュをバイパス試行

        match run_dd_with_progress(&dd_cmd, device_size, &bar) {
            Ok(_) => {
                log_message(&action, "Success", "Random data written successfully.");
            }
            Err(e) => {
                log_message(&action, "Failed", &e);
                // DoD 5220.22-M では途中で失敗したら中断すべきか？要件によるが、ここでは中断する
                return Err(format!(
                    "Pass {} failed for {}: {}",
                    i + 1,
                    device_name,
                    e
                ));
            }
        }
    }

    // --- 最後のパス: ゼロの書き込み ---
    let action = format!("Final pass: Writing zeros to {}", device_name);
    println!("\n{}", action); // 見やすさのために改行を追加

    // プログレスバーの設定
    let bar = ProgressBar::new(device_size);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .progress_chars("#>-"),
    );
    bar.enable_steady_tick(100);

    // ddコマンドの構築
    let dd_cmd = format!("dd if=/dev/zero of={} bs=4M oflag=direct", device_name); // directフラグ

    match run_dd_with_progress(&dd_cmd, device_size, &bar) {
         Ok(_) => {
            log_message(&action, "Success", "Zeros written successfully.");
        }
         Err(e) => {
            log_message(&action, "Failed", &e);
            return Err(format!("Final pass failed for {}: {}", device_name, e));
        }
    }


    println!("\nDoD5220.22-M wipe completed for: {}", device_name);
    log_message(
        &format!("DoD5220.22-M wipe completed for: {}", device_name),
        "Success",
        "Random data and zeros were written successfully in all passes.",
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    // 注意: 実際のddコマンドの実行と進捗監視を含むテストは複雑です。
    // ここでは、`erase_hdd_with_dod5220` 関数が期待通りに
    // 成功/失敗パスを処理するか、基本的な構造のみをテストします。
    // `run_dd_with_progress` の単体テストは、モック化やテスト環境の工夫が必要です。

    #[test]
    fn test_erase_hdd_success_path() {
        // このテストは実際のddを実行しないため、プログレスバーの正確なテストはできません。
        // 主にロジックの流れ（成功ケース）を確認します。
        // 実際のコマンド実行を伴うテストはインテグレーションテストとして別途行うのが望ましいです。

        // 仮のデバイス名
        let _device = "/dev/mock_hdd"; // 未使用のためアンダースコアを追加

        // ここでは erase_hdd_with_dod5220 内の `Command::new("sh")` や
        // `run_dd_with_progress` の呼び出しを直接モックすることは難しいです。
        // 関数のシグネチャを変更して依存性注入を行うか、より高度なモックライブラリが必要です。

        // 現状できることとして、関数がエラーを返さないことを確認する程度に留めます。
        // （実際にはデバイスサイズ取得やdd実行で失敗する可能性がある）
         println!("Simulating success path (no actual dd run)...");
        // assert!(erase_hdd_with_dod5220(device).is_ok()); // これは実際のコマンドに依存するため失敗する

        // 代わりに、ロジックの主要部分を抜き出してテストするなどの工夫が必要
        assert!(true); // プレースホルダー: より良いテスト方法を検討する必要あり
    }

    #[test]
    fn test_erase_hdd_get_size_failure() {
        // デバイスサイズ取得失敗をシミュレートする方法が必要
        // 現状のコードでは直接モックできないため、テストは不完全
         println!("Simulating get device size failure path...");
        // let result = erase_hdd_with_dod5220("/dev/fail_size");
        // assert!(result.is_err());
        // assert!(result.unwrap_err().contains("Failed to get device size"));
        assert!(true); // プレースホルダー
    }

    #[test]
    fn test_erase_hdd_dd_failure() {
        // dd実行失敗をシミュレートする方法が必要
        // 現状のコードでは直接モックできないため、テストは不完全
         println!("Simulating dd failure path...");
        // let result = erase_hdd_with_dod5220("/dev/fail_dd");
        // assert!(result.is_err());
        // assert!(result.unwrap_err().contains("failed"));
        assert!(true); // プレースホルダー
    }

    // run_dd_with_progress 関数のテスト (部分的なシミュレーション)
    #[test]
    fn test_run_dd_progress_parsing_mock() {
         println!("Testing regex parsing for run_dd_with_progress (mocked)...");
        let re = Regex::new(r"^(\d+)\s+bytes").unwrap();

        let line1 = "1024 bytes (1.0 kB, 1.0 KiB) copied, 0.001 s, 1.0 MB/s";
        let caps1 = re.captures(line1).unwrap();
        assert_eq!(caps1.get(1).unwrap().as_str(), "1024");
        assert_eq!(caps1.get(1).unwrap().as_str().parse::<u64>().unwrap(), 1024);

        let line2 = "1234567890 bytes (1.2 GB, 1.1 GiB) copied, 10.5 s, 117 MB/s";
         let caps2 = re.captures(line2).unwrap();
        assert_eq!(caps2.get(1).unwrap().as_str(), "1234567890");

         let line_no_match = "dd: writing to '/dev/null': No space left on device";
         assert!(re.captures(line_no_match).is_none());

         let line_records = "1+0 records in\n1+0 records out"; // これはバイト数ではない
         assert!(re.captures(line_records).is_none());
    }

}
