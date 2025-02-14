# rust_erase

<!-- [![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://example.com) -->
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`rust_erase` は、Linux環境で動作するディスク消去ユーティリティです。DoD 5220.22-M方式 (HDD) および Secure Erase (SSD) を使用して、ディスクのデータを安全かつ完全に消去します。

## 動作環境

*   **OS:** Linux (USBブートでの利用を想定)
    *   動作確認: Lubuntu 24.04
*   **Rust:** 1.75以上 (より新しいバージョンを推奨)
*   **必要なコマンド:**
    *   `lsblk`
    *   `hdparm` (SATA SSD/HDD用)
    *   `nvme-cli` (NVMe SSD用)
    *   `dd` (HDD消去用)
    *   `sudo`

## インストール

1.  **前提条件:**
    *   Rust開発環境がインストールされていることを確認してください。
    *   必要なコマンド (`lsblk`, `hdparm`, `nvme-cli`, `dd`, `sudo`) が利用可能であることを確認してください。

2.  **リポジトリのクローン:**

    ```bash
    git clone https://github.com/<your_username>/rust_erase.git
    cd rust_erase
    ```

3.  **ビルド:**

    ```bash
    cargo build --release
    ```

    ビルドされた実行ファイルは `target/release/rust_erase` に生成されます。

## 使い方

1.  **管理者権限で実行:**

    ```bash
    sudo ./target/release/rust_erase
    ```

2.  **画面の指示に従って操作:**
    *   消去するディスクを選択します。
    *   消去の確認を行います。
    *   選択したディスクの消去が開始されます。
    *   消去が完了すると、システムをシャットダウンするか、プログラムを終了するかを選択できます。

**⚠️ 警告 ⚠️**

*   **データの完全消去:** このプログラムは、選択したディスク上のすべてのデータを**復元不可能**な形で消去します。
*   **バックアップ:** 重要なデータは、必ず事前にバックアップしてください。
*   **ディスクの選択:** 間違ったディスクを選択しないように、十分に注意してください。
*   **自己責任:** このプログラムの使用によって生じたいかなる損害についても、作者は責任を負いません。

## 主な機能

*   **ディスクの自動検出:** `lsblk` コマンドを使用して、システムに接続されているディスクを自動的に検出します。
*   **ディスク情報の表示:** 各ディスクのデバイス名、モデル、タイプ (HDD/SSD)、接続タイプ (SATA/NVMe/USB) を表示します。
*   **消去方式の選択:**
    *   **HDD:** DoD 5220.22-M方式 (3回のランダムデータ書き込み + 1回のゼロ書き込み)
    *   **SATA SSD:** ATA Secure Erase (可能な場合は Enhanced Secure Erase)
    *   **NVMe SSD:** NVMe Secure Erase
*   **消去処理のログ:** `erasure_log.txt` ファイルに、消去処理の詳細なログを記録します。
*   **並列処理:** 複数のディスクが選択された場合、`rayon` クレートを使用して並列で消去処理を行います。
*   **プログレスバー:** HDD消去時に、`indicatif` クレートを使用して進行状況を表示します。(ddコマンドの出力と連携して、より正確なプログレスバーに改善予定)
*   **安全性:**
    *   消去開始前に、ユーザーに確認を求めます。
    *   消去処理中にエラーが発生した場合、ログに記録し、ユーザーに通知します。

## ライセンス

このプロジェクトは、[MIT License](https://opensource.org/licenses/MIT) のもとで公開されています。

## 貢献

バグ報告、機能提案、プルリクエストなど、歓迎します。

## TODO

*   テストの拡充 (特に外部コマンドのモック)
*   `dd` コマンドの出力と連携した、より正確なプログレスバーの表示
*   より詳細なディスク情報の取得 (SMART情報など)
*   消去方式の追加 (Gutmann方式など)
*   UIの改善 (TUI/GUI)
*   Windows/macOS対応
