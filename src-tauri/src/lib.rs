//! Tauri アプリの殻。IPC・ウィンドウ・注入・プラグインの配線のみを担う。
//! ロジックは `harubridge-core` に置くこと。依存の向きは 殻 → コア の一方向のみ。

use std::sync::atomic::{AtomicUsize, Ordering};

use tauri::{
    menu::{Menu, MenuItemBuilder, SubmenuBuilder},
    webview::{NewWindowFeatures, NewWindowResponse},
    AppHandle, Manager, Url, WebviewUrl, WebviewWindowBuilder, Wry,
};
use tauri_plugin_opener::OpenerExt;

/// ゲームの入口ページ。観測: `docs/kancolle/api/overview.md`
const GAME_ENTRY_URL: &str = "https://play.games.dmm.com/game/kancolle";

/// ゲーム画面の大きさ。観測: `docs/kancolle/api/overview.md`
const GAME_SIZE: (f64, f64) = (1200.0, 720.0);

/// ウィンドウ枠がゲーム画面から奪う高さの見込み。
///
/// Tauri は内側と外側を同じ大きさとして報告するため、枠の厚みを問い合わせる手段が無い
/// （`inner_size` / `min_inner_size` / `set_size` のいずれでも内側を指定どおりにできない）。
/// そのため見込みの定数を足す。**正確である必要はない。**
/// ずれたぶんは注入した CSS が縮小して吸収する（`injected/page-style.ts`）。
/// 合っていればゲーム画面が実寸のまま表示され、縮小も余白も生じない。
#[cfg(target_os = "macos")]
const WINDOW_CHROME_HEIGHT: f64 = 28.0;
/// `TODO(要検証)`: Windows（WebView2）での枠の厚み。実機で確認していない。
#[cfg(not(target_os = "macos"))]
const WINDOW_CHROME_HEIGHT: f64 = 0.0;

/// メインウィンドウの初期サイズ。
const MAIN_WINDOW_SIZE: (f64, f64) = (GAME_SIZE.0, GAME_SIZE.1 + WINDOW_CHROME_HEIGHT);

/// アプリ内で開いてよいポップアップのホスト。
///
/// 配信元が新しいウィンドウで開く導線（規約の表示など）を OS の既定ブラウザへ逃がすと、
/// クッキーを共有できず途切れる。ここに該当するものだけアプリ内のウィンドウで開き、
/// それ以外の外部サイトは既定ブラウザへ渡す。
const IN_APP_POPUP_HOSTS: [&str; 3] = ["dmm.com", "dmm.co.jp", "kancolle-server.com"];

/// ポップアップウィンドウのラベルを一意にするための連番。
static POPUP_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

/// 配信元ページに当てる表示調整スクリプト。
///
/// 生成物（`.js`）を埋め込む。ソース（`.ts`）とのずれは `task check:injected` が落とす。
const PAGE_STYLE_SCRIPT: &str = include_str!("../injected/page-style.js");

fn is_in_app_popup_host(url: &Url) -> bool {
    url.host_str().is_some_and(|host| {
        IN_APP_POPUP_HOSTS
            .iter()
            .any(|allowed| host == *allowed || host.ends_with(&format!(".{allowed}")))
    })
}

/// 新しいウィンドウの要求を捌く。
///
/// 配信元のものはアプリ内のウィンドウで開く。`window_features` が呼び出し元と同じ
/// WebView 設定を引き継ぐため、クッキーとセッションが共有される。
/// それ以外は OS の既定ブラウザへ渡す。**URL はログに出さない**（G-RS-43）
fn handle_new_window(
    app: &AppHandle,
    url: Url,
    features: NewWindowFeatures,
) -> NewWindowResponse<Wry> {
    if !is_in_app_popup_host(&url) {
        let _ = app.opener().open_url(url.as_str(), None::<&str>);
        return NewWindowResponse::Deny;
    }

    let label = format!("popup-{}", POPUP_SEQUENCE.fetch_add(1, Ordering::Relaxed));
    match WebviewWindowBuilder::new(app, label, WebviewUrl::External(url))
        .window_features(features)
        .build()
    {
        Ok(window) => NewWindowResponse::Create { window },
        Err(_) => NewWindowResponse::Deny,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // コアが結線されていることの暫定確認。最初の実機能が入った時点で置き換える
    let _core_version = harubridge_core::core_version();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_menu_event(|app, event| {
            let Some(main) = app.get_webview_window("main") else {
                return;
            };
            match event.id().as_ref() {
                "nav_back" => {
                    let _ = main.eval("history.back()");
                }
                "nav_forward" => {
                    let _ = main.eval("history.forward()");
                }
                "nav_home" => {
                    if let Ok(entry) = Url::parse(GAME_ENTRY_URL) {
                        let _ = main.navigate(entry);
                    }
                }
                _ => {}
            }
        })
        .setup(|app| {
            // G-RS-31 の例外: 定数の解析であり、失敗するならビルド時点で誤っている
            let entry = Url::parse(GAME_ENTRY_URL)
                .expect("GAME_ENTRY_URL は定数であり、常に URL として解釈できる");

            let main_handle = app.handle().clone();
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(entry))
                .title("Harubridge")
                .inner_size(MAIN_WINDOW_SIZE.0, MAIN_WINDOW_SIZE.1)
                // ゲーム画面より内側が小さくなると下端が欠ける。
                // ウィンドウ枠の厚みは OS とテーマで変わるため、自分で計算せず
                // 「内側がこれを下回らない」と宣言して OS に守らせる
                .min_inner_size(MAIN_WINDOW_SIZE.0, MAIN_WINDOW_SIZE.1)
                // ゲームを載せる要素は入口ページ（トップフレーム）にあるため、
                // 表示調整は全フレームに配る必要がない
                .initialization_script(PAGE_STYLE_SCRIPT)
                .on_new_window(move |url, features| handle_new_window(&main_handle, url, features))
                .build()?;

            // ゲーム画面には戻る手段が無いため、遷移を間違えると入口へ戻れなくなる。
            // 操作バー（FR-001）ができるまでの代わりとして、メニューに置く
            let back = MenuItemBuilder::with_id("nav_back", "戻る")
                .accelerator("CmdOrCtrl+ArrowLeft")
                .build(app)?;
            let forward = MenuItemBuilder::with_id("nav_forward", "進む")
                .accelerator("CmdOrCtrl+ArrowRight")
                .build(app)?;
            let home = MenuItemBuilder::with_id("nav_home", "入口ページを開き直す")
                .accelerator("CmdOrCtrl+Shift+H")
                .build(app)?;
            let navigation = SubmenuBuilder::new(app, "移動")
                .items(&[&back, &forward, &home])
                .build()?;

            let menu = Menu::default(app.handle())?;
            menu.append(&navigation)?;
            app.set_menu(menu)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
