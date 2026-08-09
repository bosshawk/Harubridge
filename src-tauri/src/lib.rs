//! Tauri アプリの殻。IPC・ウィンドウ・注入・プラグインの配線のみを担う。
//! ロジックは `harubridge-core` に置くこと。依存の向きは 殻 → コア の一方向のみ。

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // コアが結線されていることの暫定確認。最初の実機能が入った時点で置き換える
    let _core_version = harubridge_core::core_version();

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
