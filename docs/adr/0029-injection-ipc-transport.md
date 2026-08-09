# ADR-0029: 注入スクリプトから Rust への転送は、Tauri の invoke に相乗りせず自前のハンドラで受ける

- ステータス: **Proposed**
- 日付: 2026-08-03
- 決定者: プロジェクトオーナー（承認待ち）
- 関連: [ADR-0016](0016-tech-stack.md)（Tauri v2）/ [ADR-0024](0024-state-sync-granularity.md)（Rust → UI の同期）/
  [ADR-0026](archive/0026-injection-script-build.md)（注入スクリプトの作り）/
  [architecture.md](../spec/architecture.md)（「境界はページ内に置く」）

## 背景と課題

[architecture.md](../spec/architecture.md) は、観測の骨格をこう定めている。

> **全フレームに注入する。** ゲーム本体は DMM のページ内の iframe
> （`osapi.dmm.com` → `*.kancolle-server.com`）で動作している
>
> **絞り込みは注入スクリプトの中で行う。`/kcsapi/` 以外は Rust コアへ渡さない。**
> …**境界はページ内に置く**

この構造は「注入スクリプトが Rust へメッセージを送れる」ことを前提にしている。
ところが**ゲームは cross-origin の子 iframe で動く**（DMM は
`play.games.dmm.com` →（第 1 階層）`osapi.dmm.com` →（第 2 階層）ゲームサーバの 2 階層）。

**注入できることと、そこから native へ送り返せることは別の問題である。**
[ADR-0026](archive/0026-injection-script-build.md) の検討中に、Tauri のブートストラップが
main frame 限定で積まれていることが判明し、前提が成立するか不明になった。

**この前提が崩れると [ADR-0024](0024-state-sync-granularity.md) /
[0026](archive/0026-injection-script-build.md) / [0027](archive/0027-repository-layout.md) と
`architecture.md` の骨格がまとめて倒れる。** そのため先に確定させる。

## 調査結果

一次ソースを読んで確認した（2026-08-03）。

| リポジトリ | commit | バージョン |
| --- | --- | --- |
| `tauri-apps/wry` | `b95a7f8` | `Cargo.toml` version = 0.56.0 |
| `tauri-apps/tauri` | `020919a` | `crates/tauri/Cargo.toml` version = 2.11.5 |

`tauri/crates/tauri-runtime-wry/Cargo.toml:16` が `wry = "0.56.0"` で、両者は整合する。

### macOS（WKWebView）—— 送れる

**`WKScriptMessageHandler` にフレーム制限は無い。**

- `wry/src/wkwebview/class/wry_web_view_delegate.rs:99` は
  `controller.addScriptMessageHandler_name(proto_delegate, IPC_MESSAGE_HANDLER_NAME)` を
  呼ぶだけで、フレーム限定の指定が存在しない
- 同ファイル `:50-54` でハンドラは `msg.frameInfo().request().URL()` を読み、
  **そのフレームの URL** を `Request` の URI にしている。
  **サブフレームからの受信を前提にした実装である**
- main frame 限定なのは **`window.ipc` というラッパーの定義だけ**。
  `wry/src/wkwebview/mod.rs:640-645` が
  `Object.defineProperty(window, 'ipc', …)` を `for_main_only = true` で注入している。
  その中身は `window.webkit.messageHandlers.ipc.postMessage(s)` を呼ぶだけであり、
  **サブフレームからは `window.webkit.messageHandlers` を直接叩けば届く**

傍証として `wry/src/lib.rs:1174`（`with_ipc_handler` の doc）に
「**Linux / Android**: The request URL is not supported on iframes」とあり、
macOS / Windows では iframe 由来の URL が取れる想定で書かれている。

### Windows（WebView2）—— 注入は届くが、受信が届かない

- **注入は無条件で全フレームに入る。** `wry/src/webview2/mod.rs:504-506` は
  `for_main_frame_only` を**読まずに**全部 `AddScriptToExecuteOnDocumentCreated` に流す
  （ソース中のコメントも `Initialize main and subframe scripts`）。
  `wry/src/lib.rs:1004` / `:1029` にも
  「Windows: scripts are always added to subframes regardless of the
  `for_main_frame_only` option」と明記されている
- **受信は `ICoreWebView2::add_WebMessageReceived` のみ**（`wry/src/webview2/mod.rs:955`）。
  Microsoft の公式ドキュメントはこれを
  「**the top-level document** of the WebView runs `window.chrome.webview.postMessage`」
  と定義している
- **wry は iframe 用 API を一切使っていない。**
  `grep -rn 'ICoreWebView2Frame\|FrameCreated' wry/src/` のヒットは **0 件**

WebView2 側には正規の経路が存在する。

| API | 用途 | 導入 |
| --- | --- | --- |
| `ICoreWebView2_4::add_FrameCreated` | **第 1 階層の** iframe を拾う | — |
| `ICoreWebView2Frame2::add_WebMessageReceived` | そのフレームからの受信 | Runtime 1.0.1108.44 |
| `ICoreWebView2Frame7::add_FrameCreated` | **直接の子フレーム**を拾う（再帰用） | **SDK 1.0.3240.44 / 2025-05-05** |

**`ICoreWebView2_4::add_FrameCreated` だけでは足りない。**
Microsoft の概念ドキュメント `concepts/frames` は
「`CoreWebView2Frame` … is currently enabled for **top-level iframes**」と書いており、
第 1 階層しか拾わない。DMM はゲームが第 2 階層にあるため、
`ICoreWebView2Frame7::add_FrameCreated` を**再帰的に**登録してフレーム木を歩く必要がある。

> なおこの `concepts/frames` の一文は `ICoreWebView2Frame7` の出荷（2025-05）後に
> 更新されておらず、**根拠としては古い。** ただし
> 「`add_FrameCreated` は直接の子まで」という結論は下記のスペックと一致する。

必要な束縛はすべて **webview2-com 0.38**（wry の依存、`wry/Cargo.toml:68`）に存在する。

### この再帰は Microsoft 自身が示している用法である

**自作の回避策ではない。** 公式のスペックとサンプルに、そのままの形で載っている。

- **API スペック `specs/NestedFrame.md`**（`MicrosoftEdge/WebView2Feedback` PR #4982、
  merged 2025-01-07、MIT）に **「Track entire WebView2 Frames Tree」という節**があり、
  `frame7->add_FrameCreated(...)` の中で `OnFrameCreated(webviewFrame)` を
  **再帰呼び出し**する C++ / C# のサンプルが書かれている。
  Background 節も用途として **the full WebView2 frames tree** を明示している
- **公式サンプル `WebView2Samples/SampleApps/WebView2APISample/ScenarioWebViewEventMonitor.cpp`**
  （MIT、1127-1296 行）の `InitializeFrameEventView(frame, depth)` が、
  各階層で `frame2->add_WebMessageReceived`（1212 行）を張りつつ
  `frame7->add_FrameCreated` の中で `InitializeFrameEventView(frame, depth + 1)`（1267 行）を
  **再帰呼び出し**している。**本 ADR の案 A（Windows 側）はこの構造と同じである**
- **SDK 1.0.3240.44 のリリースノート**（2025-05-05 / Runtime 136）:
  「giving the app access to **all properties, methods, and events** of
  `CoreWebView2Frame` for the nested iframe」

> ⚠️ 公式サンプルには**真似してはいけない箇所がある。**
> `&m_frameCreatedToken` という単一のメンバ変数を全階層で使い回している。

### 注入が入れ子に届くことの根拠

`AddScriptToExecuteOnDocumentCreated` の文面は "all top-level document and
**child frame** page navigations" とだけ書き、深さに言及がない。
これを補う一次情報が 2 つある。

- Microsoft のエンジニアが Issue #2440 で、`FrameCreated` が入れ子で発火しない件の
  回避策として `AddScriptToExecuteOnDocumentCreated` を挙げ、
  **「which should work for nested」**と述べている
- Issue #2921 で Microsoft 側が、この API の実装が
  **CDP の `Page.addScriptToEvaluateOnNewDocument`** に由来すると明かしている。
  CDP 側の定義は **「Evaluates given script in **every frame** upon creation」**で、
  深さの制限が無い

`window.chrome.webview` が子フレームに存在することは、
**公式の JavaScript リファレンスに明記されている**:

> This will result in either the `CoreWebView2.WebMessageReceived` event
> **or the `CoreWebView2Frame.WebMessageReceived` event** being raised,
> depending on if `postMessage` is called from the top-level document
> **or from a child frame**.

### Tauri の `invoke` には相乗りできない

**これは可否ではなく、やってはいけないという判断である。**

- Tauri のブートストラップは全部 main frame 限定。
  `tauri/crates/tauri/src/manager/webview.rs:161-166` の `main_frame_script` が
  `for_main_frame_only: true` をハードコードし、`__TAURI_INTERNALS__`（`:168`）と
  invoke 初期化（`:184`）に使われる。**サブフレームには `__TAURI_INTERNALS__` も
  `__TAURI_INVOKE_KEY__` も存在しない**
- 自分で invoke key を埋めれば送れるが、`AppHandle::invoke_key()`
  （`tauri/crates/tauri/src/app.rs:1127`）の doc コメントにこうある:

  > `DO NOT expose this key to third party scripts as might grant access to the
  > backend from external URLs and iframes.`

  注入スクリプトは page world で動くため、**invoke key は DMM とゲームサーバの
  ページの JS から読める。** 本プロジェクトの側から、他人のページに
  バックエンドへの鍵を渡すことになる
- **応答が返らない。** レスポンダは `webview.eval(js)`
  （`tauri/crates/tauri/src/ipc/protocol.rs:334-336`）で、これは main frame で実行される。
  iframe 発の invoke は片道であり、main frame に「callback 未定義」のノイズが出る。
  **これは第三者が実測で報告している。**
  [tauri#6204](https://github.com/tauri-apps/tauri/issues/6204) で Tauri のメンバーが
  「WebKit は親でも子でも invoke できるオプションを持つが、
  **Webview2 (Windows) はそれをサポートせず、すべてのコールバックが親で受信される**」
  と回答しており、報告者は v1 で `window.__TAURI_IPC__` が子 iframe に無いことを確認している

## 決定

**注入スクリプトから Rust への転送は、Tauri の `invoke` を使わず、
プラットフォームごとに自前のメッセージハンドラを登録して受ける。**

`Webview::with_webview()` の中で `PlatformWebview` から native ハンドルを取り出し、
OS の API を直接呼ぶ。**wry の fork は不要**であり、Tauri / wry の公開 API で完結する。

| | 受け口 | 送り口（注入スクリプト側） |
| --- | --- | --- |
| **macOS** | `PlatformWebview::controller()` で `WKUserContentController` を取り、`addScriptMessageHandler:name:` に**自前の名前**で登録 | `window.webkit.messageHandlers.<自前名>.postMessage(...)` |
| **Windows** | `PlatformWebview::controller()` → `ICoreWebView2` → `ICoreWebView2_4::add_FrameCreated`、各フレームに `ICoreWebView2Frame2::add_WebMessageReceived` と `ICoreWebView2Frame7::add_FrameCreated` を**再帰的に**登録 | `window.chrome.webview.postMessage(...)` |

注入自体は両 OS とも全フレームに入るため、送り口は注入スクリプト側の
1 箇所の分岐で吸収できる。

### 付随して決めること

- **Tauri の `invoke` は UI（main frame）専用とする。** 注入スクリプトは触らない。
  これにより invoke key は main frame の外に一切出ない
- **受け口は「観測データを受け取る」1 種類に限る。** 注入スクリプトから
  任意のコマンドを呼べる口を作らない。ページ側の JS も同じ口を叩けるため、
  口が広いほどゲームのページから Rust を動かせる余地が広がる
- **受信したメッセージは信頼しない。** 送り元フレームの URL
  （macOS は `WKScriptMessage.frameInfo`、Windows は `ICoreWebView2Frame` の `Name`/URL）を
  照合し、想定外のオリジンからのものは捨てる。
  ただし**これは防御であって認証ではない**（同じページの JS は同じ口を叩ける）
- **Windows は Runtime のバージョンを判定し、満たさない場合は縮退する。**
  `ICoreWebView2Frame7` が無い環境では観測できない旨を表示する。
  黙って動かないのが最悪である
- **Windows は `about:blank` で起動し、ハンドラを登録してからゲームへナビゲートする。**
  `add_FrameCreated` の登録が最初のナビゲーションに間に合わないと初回フレームを取りこぼし、
  wry はナビゲーション前フックを公開していないため（詳細は「未解決事項」）
- **観測できているフレーム数をユーザーが確認できるようにする。**
  上記のバグ 2 件により、取りこぼしは黙って起きうる。
  「動いていないことに気づけない」状態を作らない

## 検討した選択肢

### 案 A: プラットフォームごとに自前のハンドラを登録する（採用）

- 利点: 絞り込みは注入スクリプト内、そこから native へ **1 ホップ**。
  `architecture.md` の構造がそのまま成立し、**同文書の変更が不要**。
  invoke key を外に出さない。wry の fork も不要
- 欠点: Windows 側に COM を直接叩くプラットフォーム固有コードが要る。
  再帰的なフレーム登録は自分で書く必要があり、取りこぼしのリスクを自分で負う

### 案 B: 子 iframe から `window.parent.postMessage` で main frame へ中継する

- 概要: 各フレームに入った注入スクリプトが親へ中継し、
  main frame でまとめて Tauri の IPC に載せる
- 利点: **native 固有コードがゼロ。** 最も安価で、両 OS で同じコードが動く。
  受信側の `event.origin` はブラウザが設定するもので偽装できないため、検証は成立する
- 却下理由: **絞り込んだ後のデータが DMM のページの JS コンテキストを 2 回通過する。**
  `postMessage` は対象ウィンドウの全リスナへブロードキャストされるため、
  DMM のページ自身のリスナからも観測データが見える。
  `architecture.md` の「境界はページ内に置く」は字面としては守られるが、
  **絞り込んだ結果をわざわざ他人のページに晒すことになり、趣旨に反する。**
  加えて `window.parent.postMessage` の発行は
  「ゲームのページに副作用を与えない」に対する新たな副作用である。
  採るなら `architecture.md` の 2 箇所の変更（＝人間の承認）が必要になる

### 案 C: wry を fork / patch して Windows 側に frame API を足す

- 概要: `ICoreWebView2Frame2::add_WebMessageReceived` を wry 本体に足す
- 利点: upstream 化の筋は通る（`with_ipc_handler` の doc が既に iframe を想定した書き方）
- 却下理由: **案 A で足りる。** Tauri が wry のバージョンを固定しているため
  `[patch.crates-io]` 運用になり、Tauri の更新に追随するコストを恒久的に負う。
  得られるものが「自前コードが wry の中に移る」だけでは釣り合わない

### 案 D: ローカル HTTP サーバ / custom protocol へ `fetch` する

- 概要: 注入スクリプトからローカルの受け口へ HTTP で送る
- **技術的には成立する。** サブフレーム発のリクエストも拾える。
  既定では out-of-process iframe を拾わないが、
  `ICoreWebView2_22::AddWebResourceRequestedFilterWithRequestSourceKinds` に
  `DOCUMENT` を渡せばよい（Microsoft のエンジニアが Issue #2341 で明示、
  および WebView2Announcements #99 の公式アナウンス）。
  **wry は既にこれを実装している**（`wry/src/webview2/mod.rs:1001-1013`）
- 却下理由: **技術的な可否ではなく、[C-02](../spec/constraints.md) の趣旨に触れるため。**
  ゲームサーバ宛てではないが、ページから新たな送信を発生させる構造になる。
  「`XMLHttpRequest` をサブクラス化し `loadend` を購読するだけ」という
  [architecture.md](../spec/architecture.md) の担保が崩れる

## 決め手

**invoke key の扱いが分岐点になった。**

案 A と案 B の差は、当初は「native コードを書くか否か」に見えた。
しかし Tauri の `invoke` に載せる限り、どの案でも
**「他人のページの JS から読める場所に、バックエンドへの鍵を置く」**ことになる。
Tauri 自身が doc コメントで禁じている使い方である。

案 A はこれを回避すると同時に、**受け口を「観測データを受け取る」1 種類に絞れる。**
ゲームのページから到達できる面が、コマンド全体から 1 つの口に縮む。
Windows の COM コードはその対価として妥当と判断した。

## 影響

- `architecture.md` は**変更不要**。案 A は既存の記述（全フレーム注入・
  ページ内で絞り込み・注入スクリプト 1 本）とそのまま一致する
- [ADR-0026](archive/0026-injection-script-build.md) の「型を Rust と共有できるか」は
  **本 ADR で確定する。tauri-specta の生成物は使えない**（IPC 経路が UI と違うため）。
  注入スクリプトと Rust の間のメッセージ型は、この 1 種類だけ別に定義する
- [ADR-0027](archive/0027-repository-layout.md) の `src-tauri/` に、
  プラットフォーム固有のモジュール（`#[cfg(target_os = ...)]`）が 1 つ増える
- Windows の動作要件に **WebView2 Runtime 1.0.3240.44 以降**（2025-05-05）が加わる。
  Evergreen Runtime は自動更新されるため実質的な障害は小さいが、配布時に明記する
- 取り消す場合のコスト: 中。案 B へ切り替えるのは注入スクリプトと
  受け口の書き換えで済むが、`architecture.md` の変更（承認）を伴う

## 未解決事項

**本 ADR は一次ソースの読解のみに基づく。Windows では実機で動かしていない。**

### 未修正の公式バグが 2 件ある（最大のリスク）

どちらも「**document が作り直されるフレーム**」で起きる同系統の穴である。

- **[WebView2Feedback#5115](https://github.com/MicrosoftEdge/WebView2Feedback/issues/5115)
  （OPEN・ラベル `bug`/`tracked`・未修正）** ——
  `FrameCreated` が入れ子 iframe で発火しないことがある。
  Microsoft 側は素直な入れ子では再帰が動くことを実測で確認している一方、
  **既存の iframe が差し替えられるケースでは自ら再現できている**
- **[WebView2Feedback#2921](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2921)
  （未修正のままクローズ）** ——
  親の script が `document.open()` / `document.write()` で作った iframe には
  スクリプトが注入されない。Microsoft 側も「still a bug、HTML を制御できない開発者には
  回避策が無い」と認めたまま `resolved temporarily` で閉じられている

`src` 属性で実 URL へナビゲートする通常の iframe には該当しない。
**DMM がそうした作りをしていないことは未確認**（下記）。

### 実機でしか分からないこと

- `TODO(要検証)`: **深さ 2 以上での実測報告が、世の中に 1 件も無い。**
  `ICoreWebView2Frame7` は 2025-05 出荷で、**再帰的に使っている第三者の実アプリが
  GitHub 上に存在しない**（ヒットするのは言語バインディングの定義のみ）。
  Microsoft の公式スペックとサンプルが示す用法ではあるが、**利用実績が事実上ゼロである**
- `TODO(要検証)`: **本プロジェクトとして Windows で一度も動かしていない。**
  macOS は Swift スパイクで実測済み（2026-08-02、注入フレーム 32、うちゲームサーバ 2）だが、
  **その結果は WKWebView のものであり Windows には転用できない**
- `TODO(要検証)`: DMM が使っているのが素の `iframe` か。
  `concepts/frames` は `frameset` / `portal` / `embed` / `fencedFrame` / `object` が
  対象外と書いている（ただしこのページ自体が Frame7 出荷後に更新されていない）
- `TODO(要検証)`: ハンドラを張っていない入れ子フレームが `postMessage` したときの挙動。
  親へバブルするのか捨てられるのかが**未文書**。
  スペックのレビューで Microsoft 側が「議論すべき」と指摘したが、merge 版に節が入らなかった

### 登録タイミング（実務上の手当てが要る）

`add_FrameCreated` は最初のナビゲーションより前に登録しないと初回フレームを取りこぼす。
ところが **wry は Windows 向けにナビゲーション前フックを公開していない**
（`WebViewBuilderExtWindows` にあるのは `with_additional_browser_args` /
`with_browser_accelerator_keys` / `with_browser_extensions_enabled` のみ）。

**したがって `about:blank`（またはローカルの初期ページ）で起動し、
ハンドラを登録してから DMM へナビゲートする**構成にして、順序を決定的にする。

### macOS 側に残るもの

- `WKScriptMessageHandler` にフレーム制限が無いことの **Apple 公式文面での裏付けは取れていない**
  （該当ページが JS レンダリングで取得できなかった）。
  根拠は wry のソースと本プロジェクトの Swift スパイクの実測である

> **Windows 対応を確定路線として扱わないこと。**
> 「Microsoft が公式スペックとサンプルで示している用法」であることは確認できたが、
> **本プロジェクトでの実測も、第三者の利用実績も無い。**

**Windows の動作確認は後回しとする**（オーナー判断 2026-08-03）。
手元は macOS であり、VM を立てて動かす手間が他の検証と桁違いに大きい。
確認項目は
**[Issue #6](https://github.com/bosshawk/Harubridge/issues/6)（W-01〜W-12）**
に整理してある。

**ただし本 ADR の結論は実測を待たずとも変わらない**（案 A 以外に選ぶ道が無い）。
実測が変えるのは「Windows 対応を確定路線と呼べるか」だけである。

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| `WKScriptMessageHandler` の登録にフレーム制限が無いこと・`frameInfo` を読んでいること | `tauri-apps/wry` `b95a7f8`（Apache-2.0/MIT）: `src/wkwebview/class/wry_web_view_delegate.rs:50-54, 99` | 2026-08-03 |
| `window.ipc` のラッパーのみ main frame 限定であること | 同上: `src/wkwebview/mod.rs:640-645` | 2026-08-03 |
| Windows で `for_main_frame_only` が無視されること | 同上: `src/webview2/mod.rs:504-506`、`src/lib.rs:1004, 1029` | 2026-08-03 |
| 受信が `add_WebMessageReceived` のみで frame API を使っていないこと | 同上: `src/webview2/mod.rs:955`、`grep -rn 'ICoreWebView2Frame\|FrameCreated' src/` = 0 件 | 2026-08-03 |
| Tauri のブートストラップが main frame 限定であること | `tauri-apps/tauri` `020919a`（Apache-2.0/MIT）: `crates/tauri/src/manager/webview.rs:161-168, 184` | 2026-08-03 |
| invoke key を iframe に晒すなという注意書き | 同上: `crates/tauri/src/app.rs:1127` の doc コメント | 2026-08-03 |
| 応答が `webview.eval` で返ること | 同上: `crates/tauri/src/ipc/protocol.rs:334-336` | 2026-08-03 |
| `add_WebMessageReceived` が top-level document 限定であること | Microsoft Learn: `ICoreWebView2::add_WebMessageReceived` | 2026-08-03 |
| `CoreWebView2Frame` が top-level iframe のみであること（**Frame7 出荷後に未更新の記述**） | Microsoft Learn: WebView2 概念ドキュメント `concepts/frames` | 2026-08-03 |
| **フレーム木全体を再帰で辿るのが公式の想定用法であること** | `MicrosoftEdge/WebView2Feedback` `specs/NestedFrame.md`（PR #4982、merged 2025-01-07、MIT）「Track entire WebView2 Frames Tree」節 | 2026-08-03 |
| **公式サンプルに再帰実装が実在すること** | `MicrosoftEdge/WebView2Samples`（MIT）: `SampleApps/WebView2APISample/ScenarioWebViewEventMonitor.cpp:1127-1296`（`add_WebMessageReceived` は 1212、再帰は 1267） | 2026-08-03 |
| 注入が入れ子に届く見込み（"should work for nested" / 実装が CDP `Page.addScriptToEvaluateOnNewDocument` 由来） | WebView2Feedback [#2440](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2440) / [#2921](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2921)（いずれも Microsoft 側の回答） | 2026-08-03 |
| `window.chrome.webview` が子フレームに存在すること | Microsoft Learn: WebView2 JavaScript リファレンス `WebView.postMessage` | 2026-08-03 |
| custom protocol がサブフレーム発を拾えること（案 D） | WebView2Feedback [#2341](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2341)（Microsoft 側の回答）/ WebView2Announcements #99 / `wry/src/webview2/mod.rs:1001-1013` | 2026-08-03 |
| Windows で invoke のコールバックが親で受信されること | [tauri#6204](https://github.com/tauri-apps/tauri/issues/6204)（Tauri メンバーの回答） | 2026-08-03 |
| `FrameCreated` の取りこぼしバグ / `document.write` 由来フレームへの未注入バグ | WebView2Feedback [#5115](https://github.com/MicrosoftEdge/WebView2Feedback/issues/5115)（OPEN）/ [#2921](https://github.com/MicrosoftEdge/WebView2Feedback/issues/2921)（未修正でクローズ） | 2026-08-03 |
| `ICoreWebView2Frame7` の導入バージョンと公開日 | Microsoft Learn（API リファレンス）/ NuGet `Microsoft.Web.WebView2` 1.0.3240.44 の `published` = 2025-05-05 | 2026-08-03 |
| 必要な COM 束縛が webview2-com 0.38 に存在すること | docs.rs `webview2-com` 0.38（wry `Cargo.toml:68` の依存） | 2026-08-03 |

> wry / tauri は dev の HEAD を clone した（release tag ではない）。
> 本件に関わる箇所は wry 0.56.0 の CHANGELOG に変更エントリが無い。
