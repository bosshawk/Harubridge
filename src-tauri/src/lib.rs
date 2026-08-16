//! Tauri アプリの殻。IPC・ウィンドウ・注入・プラグインの配線のみを担う。
//! ロジックは `harubridge-core` に置くこと。依存の向きは 殻 → コア の一方向のみ。

use tauri::{
    webview::{NewWindowFeatures, NewWindowResponse},
    Url, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tauri_plugin_opener::OpenerExt;

/// ゲームの入口ページ。観測: `docs/kancolle/api/overview.md`
const GAME_ENTRY_URL: &str = "https://play.games.dmm.com/game/kancolle";

/// メインウィンドウの初期サイズ。
/// ゲーム画面の実寸が未実測のため暫定値であり、実測後に置き換える。
const MAIN_WINDOW_SIZE: (f64, f64) = (1280.0, 800.0);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // コアが結線されていることの暫定確認。最初の実機能が入った時点で置き換える
    let _core_version = harubridge_core::core_version();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // G-RS-31 の例外: 定数の解析であり、失敗するならビルド時点で誤っている
            let entry = Url::parse(GAME_ENTRY_URL)
                .expect("GAME_ENTRY_URL は定数であり、常に URL として解釈できる");

            let handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(entry))
                .title("Harubridge")
                .inner_size(MAIN_WINDOW_SIZE.0, MAIN_WINDOW_SIZE.1)
                .on_new_window(move |url, _features: NewWindowFeatures| {
                    // 新しいウィンドウを開く遷移（`target="_blank"` / `window.open`）は
                    // アプリ内で開かず OS の既定ブラウザへ渡す。ゲーム画面は遷移しない。
                    // URL はログに出さない（G-RS-43）
                    let _ = handle.opener().open_url(url.as_str(), None::<&str>);
                    NewWindowResponse::<Wry>::Deny
                })
                .build()?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
