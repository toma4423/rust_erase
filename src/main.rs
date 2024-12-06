mod complete;
mod confirm_erase;
mod disk_selection;
mod erase_process;
mod hdd_erase;
mod logger;
mod ssd_erase;
mod title_and_description;

fn main() {
    // プログラム起動時のタイトル画面を表示
    title_and_description::show();

    // ディスク選択画面: 1つのディスクを選択
    let selected_disk = disk_selection::select_disk();

    // 選択されたディスクが存在しない場合、終了
    if selected_disk.is_none() {
        println!("No disks selected. Exiting...");
        return;
    }

    // 選択されたディスクをVec<String>に変換
    let selected_disks = vec![selected_disk.unwrap()];

    // 消去確認画面: ユーザーに確認を求める
    if !confirm_erase::confirm(&selected_disks) {
        println!("Erase cancelled. Exiting...");
        return;
    }

    // 消去プロセスの開始
    erase_process::start(&selected_disks);

    // 消去完了画面の表示
    complete::show(); // ここでcomplete::show()を呼び出す
}
