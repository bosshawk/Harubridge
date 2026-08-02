# ADR-0024: Rust コアから UI への状態同期を、ドメイン単位の全量 push と起動時の pull で行う

- ステータス: **Proposed**
- 日付: 2026-08-03
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0016](0016-tech-stack.md)（Tauri + Rust + React）,
  [ADR-0018](0018-dependencies.md)（`tauri-specta` の型付きイベント / `zustand`）,
  [ADR-0021](0021-data-persistence.md)（`state/*.json` の読み込み）,
  [architecture.md](../spec/architecture.md),
  [external/fleet-view.md](../spec/external/fleet-view.md),
  [external/timers.md](../spec/external/timers.md),
  [kancolle/api/api_port_port.md](../kancolle/api/api_port_port.md)

## 背景と課題

[architecture.md](../spec/architecture.md) の全体構造は
`S -->|IPC| U`（Rust の状態 → UI）と書いているだけで、
**何を・どの単位で・どちら向きに流すか**を決めていない。
[ADR-0018](0018-dependencies.md) は「Rust コアから UI へ状態を push する経路が主役である」ことを
`tauri-specta` 採用の決め手にしたが、その経路の形は未定のままである。実装に入る前に決める。

### 入力側の実態（実測 2026-08-02）

| エンドポイント | サイズ | 回数 / セッション |
| --- | ---: | ---: |
| `api_start2/getData`（マスタ） | 2,332,462 bytes | 1 |
| **`api_port/port`（母港一式）** | **271,824 bytes** | **7** |
| `api_get_member/require_info` | 188,731 bytes | 1 |
| `api_get_member/picture_book` | 56,707 bytes | 1 |

出典: [api_port_port.md](../kancolle/api/api_port_port.md) /
[overview.md](../kancolle/api/overview.md)（本リポジトリの実測、2026-08-02）。

**`api_port/port` は差分ではなく、母港に戻るたびに全量が飛ぶ。**
つまり **Rust コアが受け取る入力は、はじめから「スナップショット」の形をしている。**
差分にしたければ Rust 側で作り出すしかない。

### 出力側の実態（外部仕様が既に決めていること）

- **パネルごとに「最後に観測した時刻」を個別に表示する**
  （[fleet-view.md](../spec/external/fleet-view.md)「各パネルは、そのパネルの内容を
  最後に観測した時刻を `HH:MM:SS` 形式で表示する」/
  [timers.md](../spec/external/timers.md) の `最終更新`）。
- **パネル単位で縮退する。** 1 つのパネルが表示できなくても他のパネルは表示を続ける
  （fleet-view.md E-02 / timers.md E-03）。未観測のパネルは空表ではなく案内を出す（E-01）。
- **残り時間の毎秒更新は UI 側で行う。**
  timers.md は「**ゲームの通信を待たずに、表示だけを毎秒減らす**」と明記している。
  → **毎秒の IPC は要らない。** 高頻度・低遅延の経路は本アプリの要件に無い。
- **保有艦 500 隻で、絞り込み・並べ替えの変更から表示更新まで 200 ミリ秒以内**
  （fleet-view.md）。絞り込みと並べ替えのたびに IPC を挟む構造は不利になる。

### イベントの粒度と再描画の粒度は別問題である

「遠征の残り時間だけを再描画したい」は、**イベントを分ける理由にはならない。**
[ADR-0018](0018-dependencies.md) で採用した `zustand` は selector 単位で購読でき、
「It detects changes with strict-equality (old === new) by default」であり、
複数値をまとめて取るときは `useShallow` で不要な再描画を防げる
（出典: `pmndrs/zustand` README、2026-08-03 参照）。
**1 本の大きな状態を受け取っても、再描画は selector で絞れる。**

したがってイベントを分ける動機は再描画ではなく、
**縮退の単位（E-02）と最終更新時刻の単位が外部仕様でパネル単位に決まっていること**である。

### Tauri v2 の IPC の性質（一次ソースで確認）

**イベント（`emit`）— ペイロードは JS ソースとして評価される。**

公式ドキュメント（`tauri-apps/tauri-docs` `develop/calling-frontend.mdx`、2026-08-03 参照）:

> The event system is not designed for low latency or high throughput situations.

> event payloads are always JSON strings making them not suitable for bigger messages

> Under the hood it directly evaluates JavaScript code so it might not be suitable to
> sending a large amount of data.

ソースでも同じである（`tauri-apps/tauri` `dev` ブランチ、2026-08-03 参照）:

- `crates/tauri/src/event/mod.rs` — `EmitArgs::new` が `serde_json::to_string(payload)` で
  ペイロードを文字列にし、`emit_js_script` がそれを
  `(function () { const fn = window['…']; fn && fn({event: '…', payload: <JSON>}, ids) })()`
  という **JavaScript のソース文字列に直接埋め込む**
- `crates/tauri/src/webview/mod.rs` — `Webview::emit_js` がその文字列を `self.eval(...)` に渡す
- **サイズによる分岐は無い。** 何バイトであっても JS ソースの評価を通る

**コマンド（`invoke`）— カスタムプロトコル経由。JS ソース評価を通らない。**

- `crates/tauri/src/manager/webview.rs` が `ipc` という URI scheme を
  `register_uri_scheme_protocol("ipc", …)` で登録している
- 応答本体は `crates/tauri/src/ipc/mod.rs` の
  `enum InvokeResponseBody { Json(String), Raw(Vec<u8>) }`
- 公式ドキュメント（`develop/calling-rust`）は戻り値の JSON 直列化について
  「This can slow down your application if you try to return a large data such as a file or
  a download HTTP response」と述べ、大きなバイナリには `tauri::ipc::Response` を勧めている

**`Channel` — 8 KB を境に経路を切り替える。**

`crates/tauri/src/ipc/channel.rs` に閾値と、上流自身の計測コメントがある。

```rust
/// Maximum size a JSON we should send directly without going through the fetch process
// 8192 byte JSON payload runs roughly 2x faster through eval than through fetch on WebView2 v135
const MAX_JSON_DIRECT_EXECUTE_THRESHOLD: usize = 8192;
```

閾値を超えると `eval` をやめ、フロントエンドから
`plugin:__TAURI_CHANNEL__|fetch` を叩かせて IPC のカスタムプロトコル経由で取りに行かせる。
**上流が「8 KB を超えたら eval は不利」と判断しているのに対し、`emit` にはこの切り替えが無い。**

**参考計測（代理値。WebView での実測ではない）**

艦 400 隻を模した 227,527 bytes の JSON を、(a) JS ソースとして `eval` する場合と
(b) `JSON.parse` する場合で比較した（Node.js v26.1.0 / V8、100 回平均、本リポジトリで実施 2026-08-03）。

| | 1 回あたり |
| --- | ---: |
| (a) JS ソースとして eval | **2.3 ms** |
| (b) `JSON.parse` | **0.45 ms** |

比は約 5 倍だが、**絶対値は 7 回で 16 ms 程度にすぎない。**
`TODO(要検証)`: WKWebView / WebView2 での実測ではなく、Node.js 上の代理計測である。

→ **IPC のバイト数は、本決定の決め手にならない。**
決め手は性能ではなく、**真実の情報源をどこに置くか**である。

### `tauri-specta` v2 でイベントを増やすコスト（rc.25 の実物で確認）

`tauri-specta@=2.0.0-rc.25` のクレート実体を読んで確認した（2026-08-03）。

**1 イベントを増やすのに要る編集は Rust 側 2 箇所だけである。**

| # | 場所 | 内容 |
| --- | --- | --- |
| 1 | 型の定義 | `#[derive(Serialize, Deserialize, Clone, Type, Event)]` を付ける（定義と同じ場所） |
| 2 | `Builder::events(collect_events![…])` | 1 行足す |
| 3 | TypeScript 側 | **不要。** `bindings.ts` が再生成され、`events.<名前>.listen(cb)` が型付きで生える |

イベント名は構造体名の kebab-case が既定（`#[tauri_specta(event_name = "…")]` で上書き可）。
`listen` は `@tauri-apps/api/event` の `listen` をそのまま返すため、
戻り値は `Promise<UnlistenFn>` である（React では `useEffect` の cleanup で自分で解除する）。

**RC 起因の制約（イベントを増やすほど面が広がるもの）:**

- **失敗が panic である。** `mount_events` の呼び忘れ、`collect_events!` への登録漏れ、
  イベント名の重複は、いずれも panic する。
  ペイロード型が named type でない場合も
  `Can't register event {} with non-reference type` で panic する
  （`Vec<Ship>` のような型をそのままペイロードにできない）。
  上流の Issue #220「Remove all `panic`'s」は open のままである
- `Builder::events` の doc に
  「**WARNING: This method will overwrite any previously registered events.**」とあり、
  分割登録できない。全イベントを 1 箇所にまとめて渡すことになる
- TypeScript 側に `emitTo` / `listenAny` が無い（Issue #187 / PR #249 が未マージ）。
  **本アプリは単一ウィンドウなので影響しない**
- `Channel` に型が付くのは**コマンドの引数と戻り値だけ**である。
  イベントのペイロードに `Channel` を含める用途は未サポートで、検出もされない（Issue #198、open）

**評価**: 登録漏れ・名前重複は**起動時に決定的に panic する**ため、
開発中に必ず表に出る。C-03 が求める「未知の入力でも停止しない」縮退とは別の話であり、
これは受け入れられる。一方で `Builder::events` の上書き挙動と panic の存在は、
**イベントを無制限に増やす設計を支持しない。**

### 既存の専ブラがどうしているか（実際のコードで確認、2026-08-03）

| ツール | イベントの単位 | 全量 / 差分 | 真実の情報源 | 毎秒の残り時間 |
| --- | --- | --- | --- | --- |
| **poi**（Electron / TS） | 中継は 1 本（`network.on.response`）→ Redux で API パス別 action | action は当該レスポンスの全量。reducer で `compareUpdate` によりマージ | **UI 側の Redux store** | レンダラの単一 `ticker`。**ストアに書き戻さない** |
| **ElectronicObserver**（C#） | グローバル 1 本 + **API パス別に約 90 本** | コアが in-place 更新（`api_port/port` の艦は `Clear()` して全再構築） | **コア側の `KCDatabase.Instance`** | `FormMain` の 1 本のタイマー（1000 ms）→ `SystemEvents.UpdateTimerTick` |
| **KancolleSniffer**（C#） | イベントを持たない。`Sniff()` の**戻り値のビットフラグ**（ドメイン別 10 種） | **データを運ばない。** UI がコアからプルする | **コア側の `Sniffer` インスタンス** | `Main._mainTimer`（1000 ms）→ 対象パネルを総なめ |
| **FUSOU**（**Tauri / Rust / SolidJS**） | **ドメイン別に約 30 のイベント名**（`set-kcs-ships` など）× `Add` / `Set` / `Identifier` | **`Set` = 全量、`Add` = 差分**を型で分ける | Rust のグローバルとフロントの store の**二重** | **存在しない**（未実装） |

出典（いずれも 2026-08-03 に実際のソースを読んだもの）:

| ツール | リポジトリ / 読んだ地点 | ライセンス | 主に読んだファイル |
| --- | --- | --- | --- |
| poi | `poooi/poi` master `a515460` | MIT | `lib/game-api-broadcaster.ts`, `views/env-parts/data-resolver.ts`, `views/redux/info/*.ts`, `views/utils/tools.ts`, `views/components/main/parts/countdown-timer.tsx` |
| ElectronicObserver | `andanteyk/ElectronicObserver` develop `469015b` | MIT | `Observer/APIObserver.cs`, `Observer/APIBase.cs`, `Observer/kcsapi/api_port/port.cs`, `Data/KCDatabase.cs`, `Utility/SystemEvents.cs`, `Window/FormDock.cs` |
| KancolleSniffer | **`fujieda/KancolleSniffer`** master `daa8805` | **Apache-2.0** | `Sniffer.cs`, `Main.cs`, `MainWindow.cs`, `View/UpdateContext.cs`, `View/MainWindow/NDockPanel.cs`, `Model/AlarmTimer.cs` |
| FUSOU | `tsukasa-u/FUSOU` main `a27a905` | MIT | `packages/FUSOU-APP/src-tauri/src/json_parser.rs`, `src/cmd/tauri_cmd.rs`, `packages/kc_api/crates/kc-api-interface/src/interface.rs`, `packages/FUSOU-APP/src/utility/provider.tsx` |

読み取れたこと:

1. **Tauri で書かれた前例（FUSOU）は、ドメイン別のイベント名で全量を `emit_to` している。**
   `handle.emit_to("main", "set-kcs-ships", data)` の形で約 30 種。`Channel` は使っていない。
   `invoke` は「引き直し」専用で、**戻り値を返さず結果を `emit_to` に流す**
2. **4 者とも残り時間を状態として持たない。** 保持するのは絶対完了時刻で、
   毎秒の tick は「完了時刻 − 現在時刻」を描画時に計算し直すだけである。
   **毎秒タイマーはアプリ全体で 1 本**（poi の `ticker`、EO の `UIUpdateTimer`、KS の `_mainTimer`）
3. **状態を更新してから通知する順序が全員固定されている。**
   EO は `base.OnResponseReceived()` を最後に呼び、FUSOU は `restore()` / `add_or()` の後に `emit_to` する
4. **真実の情報源をコア側に置くのは EO と KS。** poi は UI 側（Redux）だが、
   poi は main も renderer も JavaScript であり、
   **main 側に状態を置く必然が無い**構成である。本プロジェクトはその条件を満たさない
5. **EO と KS は「変わったという通知」だけを送り、UI がコアから読み直す。**
   ただし両者とも**単一プロセス**であり、「読み直す」は関数呼び出しである（下記 案 D'）

## 決定

**真実の情報源（source of truth）は Rust コアに置く。UI 側の store はキャッシュであり、正ではない。
Rust → UI は「ドメイン単位の型付きイベントで、そのドメインの現在値を全量 push」する。
差分イベントは送らない。起動直後の初期状態だけは UI から `invoke` で pull する。**

具体的には次のとおり。

1. **ドメインの境界は、外部仕様のパネルに合わせる。**
   判断基準は「**最終更新時刻を別々に表示する単位か**」「**E-02 で片方だけ縮退させる単位か**」。
   この 2 つが分かれるものは別ドメインにする。それ以外の理由でイベントを増やさない
2. **各イベントのペイロードは、そのドメインの現在値の全量**（＋そのドメインを観測した時刻）とする。
   差分・パッチ・変更フラグを送らない
3. **変化していないドメインは送らない。** `api_port/port` が全量で 7 回来ても、
   前回送った値と同じドメインは emit しない。
   これは「差分を送る」ことではなく「**送信を抑止する**」ことである
4. **初期状態は pull で埋める。** UI はマウント時にドメインごとの取得コマンドを `invoke` する。
   Rust は起動時に `state/*.json`（[ADR-0021](0021-data-persistence.md)）を読んで
   メモリ上の状態を組み立てておく。**未観測は「空」ではなく未観測として表現し、
   E-01 を UI が判定できるようにする**
5. **マスタデータ（`api_start2/getData`）を UI に渡さない。**
   艦名・装備名の解決、および制空値などの計算は Rust 側で済ませ、
   UI には表示に必要な形の状態だけを渡す
6. **残り時間を状態として持たない。** Rust が渡すのは**絶対完了時刻**であり、
   残り時間は UI が描画時に計算する。毎秒の tick は UI 側に**アプリ全体で 1 本**だけ置く
   （既存 3 実装がすべてこの形である。上記「読み取れたこと」2）
7. **表示都合の派生は UI 側で行う。** 絞り込み・並べ替えは UI 内で完結させ、IPC を挟まない
8. **Rust 側の状態を更新し終えてから emit する。**
   イベントを受けた UI が、Rust に問い合わせ直しても矛盾しない状態を保つ
   （既存 3 実装がすべてこの順序である。上記「読み取れたこと」3）
9. **`tauri::ipc::Channel` を使わない**（下記 案 E）

## 検討した選択肢

### 案 A: ドメイン単位の全量 push ＋ 起動時 pull（採用）

- 概要: Rust が状態を持ち、ドメインごとに型付きイベントで全量を push する。
  初期状態のみ `invoke` で取りに行く
- 利点:
  - **UI 側にマージ処理が生まれない。** 受け取ったものをそのまま置き換えるだけであり、
    [architecture.md](../spec/architecture.md) の
    「UI は Rust コアから受け取った状態のみを描画する」がそのまま成り立つ
  - **入力の形（全量スナップショット）と出力の形が一致する。** 変換を挟まない
  - 縮退の単位（E-02）と最終更新時刻の単位が、イベントの単位とそろう
  - 起動直後・ゲーム画面を開く前でも `state/*.json` の内容を表示できる。
    `api_port/port` は母港に戻るまで飛ばないため、**pull が無いと起動直後に何も出せない**
  - **Tauri での前例がある。** FUSOU が `emit_to("main", "set-kcs-<ドメイン>", data)` の形で
    ドメイン別に全量を送っている
- 欠点:
  - 変化していない部分も含めてドメイン全量を送る。`api_port/port` 由来の保有艦のように
    大きく、かつ毎回わずかに変わるドメインでは、削れるバイト数が小さい
  - イベントの数だけ `collect_events!` の登録と TypeScript 側の購読が増える

### 案 B: 全状態を 1 本のイベントで全量 push する

- 概要: ドメインを分けず、アプリの状態全体を 1 つの型にまとめて 1 本のイベントで送る
- 利点:
  - イベントが 1 本なので、`collect_events!` の登録漏れ・名前重複という
    `tauri-specta` の panic 面が最小になる
  - 「どのイベントが来たら何を更新するか」という対応表が要らない
- 欠点:
  - 外部仕様がパネルごとの最終更新時刻と E-02 の個別縮退を要求しているため、
    **1 本にしてもペイロードの内側でドメインに分ける必要が残る**
  - `api_port/port` と無関係な更新（図鑑・プリセットなど）でも状態全体を送ることになり、
    `emit` はサイズによらず常に JS ソース評価の経路を通る
  - **調べた 4 実装のいずれも 1 本にまとめていない。**
    poi は API パス別の action、EO は API パス別イベント、KS はドメイン別ビットフラグ、
    FUSOU はドメイン別イベント名である
- 却下理由: 分割は外部仕様が要求しており、1 本にしても内側で分けるだけで利点が残らない。

### 案 C: 差分イベント（変わった部分だけを送る）

- 概要: Rust 側で前回の状態と比較し、変化した部分だけをイベントで送る
- 利点:
  - IPC のバイト数が最小になる
  - **FUSOU が実際に採っている**（`Add::Ships` が `Option` 化されたフィールドを送り、
    フロント側が `null` でないフィールドだけをパス指定で書き込む）
- 欠点:
  - **入力が全量である。** `api_port/port` は毎回全量で来る（実測 2026-08-02）ため、
    差分は Rust 側で前回値との比較から作り出すしかない
  - 作った差分を当て直す**マージ処理が UI 側に必要になる。**
    その時点で UI の store は「Rust の状態のコピー」ではなく
    「UI が独自に組み立てた状態」になり、**真実の情報源が 2 つになる。**
    FUSOU が実際にそうなっており、Rust 側のグローバルとフロント側の store の
    両方が状態を持っている
  - 取りこぼし（listen 登録前に emit された、途中で失敗した）からの復帰に、
    再同期の経路をもう 1 本作る必要がある
- 却下理由: 節約できるのは 1 回あたり数ミリ秒（上記の代理計測）であり、
  その対価に UI 側へ第二の状態機械を作るのは釣り合わない。

### 案 D: pull のみ（UI が定期的に `invoke` で取りに行く）

- 概要: push を持たず、UI がポーリングして状態を取得する
- 利点:
  - `invoke` はカスタムプロトコル経由で応答するため、
    **JS ソース評価を通らない。** 大きなペイロードには `emit` より素直である
  - イベントの登録が要らず、`tauri-specta` の panic 面が消える
- 欠点:
  - **更新の契機を知っているのは Rust 側だけである**（XHR を観測するのは Rust コア）。
    ポーリングにすると、間隔ぶん表示が遅れるか、変化が無いときも取りに行くことになる
  - 完了通知（FR-023 / FR-024）は Rust 側で出すため、**Rust はどのみち変化を検知している**
- 却下理由: 変化を知っている側が黙り、知らない側が定期的に尋ねる構造になるため。
  ただし **pull の経路自体は採用案でも残す**（初期状態の取得に使う）。

### 案 D': 「変わった」という通知だけを送り、UI が `invoke` で取りに行く（ElectronicObserver / KancolleSniffer 方式）

- 概要: イベントはドメイン識別子だけを運び、データを載せない。
  UI は通知を受けてから、そのドメインの取得コマンドを `invoke` する
- 利点:
  - **本数のうえで最も前例がある。** 調べた 4 実装のうち EO と KS がこの形である。
    KS の `Sniffer.Update` は 10 種のビットフラグで、
    `MainWindow.UpdateInfo` が立っているビットのパネルだけを再描画する
  - 大きなペイロードが `invoke`（カスタムプロトコル）を通り、
    **JS ソース評価の経路を避けられる**
  - **UI が開いていないパネルのデータを送らずに済む**
- 却下理由: **EO と KS は単一プロセスであり、「取りに行く」が関数呼び出しである。**
  Rust ↔ WebView では IPC の往復になり、通知 1 回につき `invoke` が 1 回増える。
  経路が 2 本になり、通知と取得の間に競合の窓も開く。
  得られる利点（eval の回避）は代理計測で 1 回あたり 2 ms 台であり、現時点で釣り合わない。
  なお「開いていないパネルを送らない」という利点は本物であり、
  下記「未解決事項」で再検討の対象として残す。

### 案 E: `tauri::ipc::Channel` で状態を流す

- 概要: UI からコマンド経由で `Channel` を渡し、Rust がそこへ状態を送り続ける
- 利点:
  - **8 KB を超えると `eval` をやめて fetch 経路に切り替わる**
    （`crates/tauri/src/ipc/channel.rs` の `MAX_JSON_DIRECT_EXECUTE_THRESHOLD`）。
    大きなスナップショットでは `emit` より速い可能性が高い
  - 順序が保証される。公式ドキュメントも
    「Channels are designed to be fast and deliver ordered data」と述べている
  - `tauri` の `specta` feature が `Channel` の `Type` 実装を持つため、
    **コマンドの引数として渡す限り型は付く**
- 却下理由: `Channel` はコマンド呼び出しに紐づく一方向ストリームであり、
  張り直しの責務が UI 側に生まれる。
  **かつ 1 ドメインあたりのペイロードが 8 KB の閾値をどれだけ超えるかを計測していない。**
  Tauri 製の前例である FUSOU も `Channel` を使っていない。
  複雑さを先に払う根拠が現時点で無い。

### 案 F: マスタデータを UI に渡し、名前解決と計算を UI 側で行う

- 概要: `api_start2/getData` を UI に渡し、艦名・装備名の解決や制空値の計算を TypeScript で行う
- 利点:
  - 状態イベントのペイロードから名前が消え、艦 1 隻あたりのバイト数が減る
  - **FUSOU が実際にそうしている**（`set-kcs-mst-ships` / `set-kcs-mst-slot-items` など、
    マスタもドメインとしてフロントへ emit している）
- 却下理由:
  - **2,332,462 bytes（実測 2026-08-02）である。** `emit` で渡せば必ず JS ソース評価を通り、
    `invoke` で渡しても UI 側に 2.3 MB が常駐する
  - E-03（未知の艦娘を `不明 (ID: 1234)` と表示する）はマスタ参照の結果であり、
    [architecture.md](../spec/architecture.md) は
    「縮退はパース層で行い、それより内側へ不完全なデータを流さない」としている。
    UI 側で解決すると縮退が UI に移る
  - [ADR-0016](0016-tech-stack.md) の決め手（信用できない入力を型で扱えることを理由に
    Rust を選んだ）と逆行する

## 決め手

**UI をキャッシュに留め、真実の情報源を Rust に一本化するために、
IPC のバイト数の最小化（差分送信）を捨てた。**

## 影響

- 実装への影響:
  - Rust コアはメモリ上に状態を保持し、起動時に `state/*.json` から復元する
    （[ADR-0021](0021-data-persistence.md) の読み出し経路と同じもの）
  - ドメインごとに「型 + イベント + 取得コマンド」の 3 点が対になる。
    `collect_events!` と `collect_commands!` の 2 つのリストに 1 行ずつ足す
  - UI 側は「取得コマンドで初期化 → イベントで置き換え」の 1 パターンで統一される
  - 毎秒の tick を UI 側に 1 本だけ置く。個々のコンポーネントが `setInterval` を持たない
- ドキュメントへの影響:
  - [architecture.md](../spec/architecture.md) の「全体構造」に、
    Rust コアと UI の間の粒度の原則を書き足す必要がある。
    **`architecture.md` の変更には人間の承認が要る**ため、本 ADR が `Accepted` になってから行う
  - [ADR-0018](0018-dependencies.md) が `tauri-specta` を選んだ理由（型付きイベント）は、
    本 ADR で初めて具体的な使い方が決まる。**ADR-0018 は書き換えない**
- 取り消す場合のコスト: **低〜中。** イベントの粒度の変更は
  Rust の型と UI の購読の付け替えで済み、保存形式（[ADR-0021](0021-data-persistence.md)）や
  観測方式（[ADR-0016](0016-tech-stack.md)）には波及しない。
  ただし「真実の情報源を Rust に置く」を覆すと UI 側にマージ処理が必要になり、コストは高い

## 未解決事項

- `TODO(要検証)`: **`emit` の実測。** 上記の 2.3 ms は Node.js 上の代理計測であり、
  WKWebView / WebView2 で 200 KB 級のペイロードを `emit` したときの実測ではない。
  **案 E（`Channel`）と 案 D'（通知 + pull）を再検討するかどうかはこの実測に依る。**
  再検討の条件を先に定めておく: **1 ドメインのペイロードが恒常的に 100 KB を超え、
  かつ実測で 1 回あたり 50 ms を超えるなら、そのドメインだけ経路を変える**
- `TODO(要検証)`: **Rust で解釈したあとの状態が何バイトになるか。**
  `api_port/port` の 271,824 bytes は生の JSON であり、
  不要フィールドを落とせば減り、マスタから名前を埋めれば増える。**どちらに転ぶか未計測**
- `TODO(未確定)`: ドメインの具体的な割り方。
  境界の**決め方**は本 ADR で決めたが、実際の一覧はコードが正とする
  （[ADR-0008](0008-code-as-source-of-truth.md)）
- `TODO(未確定)`: 「未観測」をペイロード上どう表現するか（`Option` か専用の列挙か）。
  E-01 / E-02 / E-03 の区別が UI 側で付けばよく、形は実装で決める
- `TODO(未確定)`: UI 側が listen を張る前に emit された場合の扱い。
  採用案では「マウント時に pull してから listen を張る」で埋まる見込みだが、
  その間に起きた更新を取りこぼす窓が残る。実装時に閉じ方を決める
- `TODO(未確定)`: 大きく、かつ滅多に開かれないパネル（図鑑・保有装備など）を、
  push の対象から外して開いたときだけ pull するかどうか（案 D' の部分適用）。
  本 ADR は全ドメインを push する前提で書いているが、
  1 ドメインあたりのバイト数が判明した時点で再検討しうる
- `TODO(未確定)`: 通知（FR-023 〜 FR-025）の判定を Rust と UI のどちらで行うか。
  本 ADR は「完了時刻は絶対時刻で渡し、残り時間は UI が計算する」までしか決めていない。
  KancolleSniffer は `TimeStep`（前回 tick と今回 tick の 2 点）で
  「またぎ」を判定して取りこぼしを防いでいるが、本プロジェクトでどちら側に置くかは未定
