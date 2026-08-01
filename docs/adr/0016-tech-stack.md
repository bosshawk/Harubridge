# ADR-0016: 技術構成を選定する

- ステータス: **Proposed**
- 日付: 2026-08-02
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0004](0004-defer-tech-stack-decision.md)（本 ADR が採択されれば置き換える）,
  [ADR-0003](0003-agent-driven-development.md), [ADR-0011](0011-license-mit.md),
  調査: Issue #1 / #2

## 背景と課題

オーナーから示された要件:

1. **最低限 macOS で動作すること。Windows でも動作すること**
2. **ローカルの処理性能を重視する**
3. Rust / Go / Kotlin / Web 系（React など）を比較対象に含めること

### 前提の訂正: 本文取得能力は選定基準にならない

本 ADR の初版では「`/kcsapi/` のレスポンス本文を取得する API があるか」でランタイムを絞り込み、
Rust と Kotlin を実質的に排除していた。**これは誤りである。**

「Chromium が必要」と「CEF バインディングが必要」を混同していた。
**ローカル MITM プロキシを自前で持つなら、ブラウザ側に本文取得能力は要らない。**
ブラウザに要求されるのは次の 2 点だけである。

- プロキシを向けられること（`--proxy-server` 相当）
- **自前の CA を、そのブラウザにだけ信頼させられること**

後者は次の手段で満たせる（いずれも文書化された機能）。

- Electron: `session.setCertificateVerifyProc` で自前 CA をアプリ内でのみ許可
- 単体起動の Chromium: `--ignore-certificate-errors-spki-list=<hash>` で特定証明書のみ信頼

→ **OS の信頼ストアに触れずに済む**という利点は Electron 固有ではない。
→ CEF の filter API も CDP も WKWebView の制約も、**プロキシ方式では判断材料にならない。**

### では何が判断軸か

**Chromium の配布・更新・コード署名を誰が引き受けるか。**

| 解決者 | 手段 | 使える言語 |
| --- | --- | --- |
| Electron | ランタイム同梱 | TypeScript / JavaScript |
| **JetBrains** | **JCEF（JetBrains Runtime 同梱）/ KCEF** | **Kotlin / Java** |
| Energy | CEF バイナリ同梱 | Go |
| **自分** | Chromium を自前同梱、またはユーザーの Chrome を起動 | 任意（Rust を含む） |

macOS の署名・公証を含む Chromium の配布は、単独開発で背負うには重い。
**これが実質的な選定軸である。**

## 決定

以下を採用する。

| レイヤ | 採用 |
| --- | --- |
| ブラウザランタイム | **Electron** |
| 観測方式 | **アプリ内 MITM プロキシ**（自前 CA を同梱ブラウザにのみ信頼させる） |
| コア言語 | **TypeScript** |
| UI | **React** |

**OS の信頼ストアには触れない。** ユーザーに CA 証明書のインストールを求めない。

## 検討した選択肢

### 案 A: Electron + TypeScript + React（採用）

- 利点:
  - **poi が同一構成で長期運用されている**（Electron + React + Redux + TypeScript, MIT）。
    艦これという対象に対して構成が成立することが実証済み。`lib/proxy.ts` を持つ構成から
    プロキシ方式も同型と推測される（`TODO(要検証)`: 実装未読）
  - Chromium の同梱・更新・macOS の署名／公証まで、`electron-builder` 系の前例が厚い
  - 情報量が最も多く、**エージェントがすべて実装する体制
    （[ADR-0003](0003-agent-driven-development.md)）に最も適合する**
- 欠点:
  - 言語が TypeScript に固定される
  - 配布サイズが大きい

### 案 B: Kotlin + Compose Multiplatform + JCEF / KCEF（**有力な対抗**）

- 概要: UI を Compose Multiplatform、ゲーム画面を JCEF、プロキシを JVM 上で実装。**全レイヤが Kotlin。**
- 利点:
  - **JCEF は JetBrains Runtime に同梱され、IntelliJ 系 IDE で本番運用されている。**
    Chromium の配布・保守を JetBrains が引き受けており、
    **案 A と同じ条件を満たす唯一の非 TypeScript 選択肢**
  - Kotlin の型システムと、JVM の並行処理・永続化ライブラリは TypeScript より強い
  - `compose-webview-multiplatform` が 1.3.0 で JCEF、1.7.0 で KCEF に移行しており、
    Compose との統合に前例がある
- 懸念（`TODO(要検証)`）:
  - **KCEF が実行時に Chromium をダウンロードする挙動を取るか。**
    取る場合、初回起動体験と配布方式に影響する。**採用するなら最初に確認すべき点**
  - JCEF には `getResourceResponseFilter` が存在しない（一次ソースで確認済み）。
    ただし**プロキシ方式を採る本 ADR では影響しない**
  - 艦これ界隈での採用実績が確認できない。日本語情報も少ない
- 却下理由: 上記の懸念そのものではなく、**案 A に対する優位が「言語の質」に留まる**ため。
  ドメイン固有の前例（poi）を持つ案 A の実装可能性の高さを、
  [ADR-0003](0003-agent-driven-development.md) の体制では上回れないと判断した。

### 案 C: Go + Energy（CEF 同梱フレームワーク）

- 概要: CEF バイナリを同梱する Go 製フレームワーク。Apache 2.0。
  Windows / **macOS（Intel・Apple Silicon 両対応）** / Linux。
  ドキュメントは中国語・英語の両方があり、レンダリングに LCL / CEF / Webview を選べる
- 利点: Chromium の配布を Energy が引き受ける（案 A / 案 B と同じ条件）。
  単一バイナリに近い配布とネイティブの処理性能
- 却下理由:
  - **star 607、fork 48 と、案 A / 案 B に比べてコミュニティ規模が小さい。**
    [ADR-0003](0003-agent-driven-development.md) の体制では参照できる前例の量が
    そのまま実装可能性に効く
  - ネイティブ UI が **LCL（Lazarus / Free Pascal 由来のコンポーネントライブラリ）**に依存し、
    Go → LCL → CEF というビルドチェーンになる。トラブル時の切り分けが難しい
  - 艦これ界隈での採用実績が確認できない

### 案 C': Go + 別プロセス Chromium（`chromedp` + `goproxy`）

**Go を選ぶなら、案 C ではなくこちらが最善形である。**

- 概要: MITM プロキシと UI 配信を Go で実装し、ゲーム画面は別プロセスの Chromium に任せる。
  ズーム・スクリーンショット・ミュートは CDP 経由で制御する
- **ライブラリが両方とも枯れている**:
  - `elazarl/goproxy` — 10 年の歴史があり、production 利用可能と作者が明言。
    **TLS 証明書を生成する HTTPS MITM を正面から機能として持つ**。本用途に直球で適合する
  - `chromedp` — star 13,169、MIT。CDP クライアントとして成熟
- 利点:
  - **プロキシを書く言語として Go は最も適した部類。** 標準ライブラリだけで
    TLS・HTTP・証明書生成が完結する
  - 配布物が小さい単一バイナリ。常駐メモリも Electron / JVM より軽い
  - UI はローカル Web サーバで React を配信すれば、フロントエンドの選択肢も残る
- 却下理由:
  - **Chromium の配布を自分で背負う**（同梱するかユーザーの Chrome を使うか。後述の案 D' 参照）
  - プロセスが 2 つに分かれ、ウィンドウ統合とライフサイクル管理が複雑になる
  - 艦これ界隈での採用実績が確認できない

### 案 D: 任意の言語（Rust / Go / Kotlin）＋ 別プロセスの Chromium ＋ CDP 制御

- 概要: MITM プロキシと UI を任意の言語で書き、ゲーム画面は別プロセスの Chromium に任せる。
  ズーム・スクリーンショット・ミュートは CDP（`--remote-debugging-port`）経由で制御する。
  **言語の制約が完全に消える構成。**
- 利点: 言語が自由。**Rust もこの構成なら成立する**
  （CEF バインディングを使わないため、バインディングの断片化を回避できる）
- 却下理由:
  - **Chromium の配布を自分で背負う。** 同梱すればサイズ・更新・macOS の公証を
    自前で解決する必要があり、ユーザーの Chrome を使えばバージョン差異と
    インストール前提という別の問題が生じる
  - プロセスが 2 つに分かれ、ウィンドウ統合とライフサイクル管理が複雑になる
  - **この複雑性に見合う必要が現時点で示されていない**

### 案 D': 任意の言語 ＋ **ユーザーの Chrome** を起動する

- 概要: 案 D / 案 C' の変種。Chromium を同梱せず、**ユーザーが既に持っている Chrome** を
  専用プロファイル（`--user-data-dir`）で起動し、`--proxy-server` と
  `--ignore-certificate-errors-spki-list` を与える
- 利点:
  - **Chromium の配布問題が消える。** 配布物は数 MB のバイナリ 1 つで済む
  - 対象が「自分と身近な数人」（[overview.md](../spec/overview.md)）であれば、
    Chrome の導入を前提にしても現実的
- 却下理由:
  - **Chrome のインストールを前提にする**という外部依存が増える
  - ユーザーの Chrome が自動更新されるため、**バージョン差異を制御できない**。
    [C-03](../spec/constraints.md)（非公開仕様への依存）に加えて、
    ブラウザ側の不確実性も抱えることになる
  - 「専用ブラウザ」という [overview.md](../spec/overview.md) の位置づけから外れる

### 案 E: Rust + CEF バインディング

- 却下理由: **Rust の CEF バインディングは断片化しており、本命クレートが存在しない。**
  複数プロジェクトが並立し、「積極的にサポートしない」「多くが未実装で十分にテストされていない」と
  作者自身が明記しているものが複数ある。
  なお **Rust を採るなら案 D の構成を選ぶべきで**、この案は Rust の最善形ではない。

### 案 F: Tauri / wry（OS の WebView）

- 却下理由: プロキシ方式なら macOS でも成立しうるが、
  **CA を信頼させる手段が OS の WebView では限定的**であり、
  結果として OS の信頼ストアへの証明書導入が必要になる可能性が高い（`TODO(要検証)`）。
  FUSOU が Tauri でプロキシ方式かつ macOS 非対応である事実と整合する。

### 案 G: ブラウザ拡張 + ローカル中継

- 却下理由: `chrome.devtools.network` は **DevTools を開いていないと発火しない**（Issue #2）。
  常用する専ブラとして成立しない。Manifest V3 移行で KC3改 が動作不能になった前例もある。

## 性能について

**言語選択では支配的コストは変わらない。**

計算コストの大半は **Chromium によるゲーム画面の描画**であり、どの案でも同じ Chromium が担う。
観測処理は 1 リクエストあたり数十 KB の JSON パースと集計であり、
TypeScript で体感差が出る規模ではない（`TODO(要検証)`: 実測していない）。

差が出うるのは常駐メモリ・起動時間・大量の履歴集計だが、
**Chromium を抱える時点で大半が決まる。**
したがって**性能のみを根拠に案 B / C / C' / D を選ぶことはできない。**

ただし各案には性能以外の固有の価値がある。

- **案 B（Kotlin）**: 型と言語の質。JVM の並行処理・永続化ライブラリ
- **案 C'（Go）**: **プロキシ実装への適合度**。`goproxy` は HTTPS MITM を正面から
  機能として持ち、本用途に直球で当たる。配布物と常駐メモリも最小

## 決め手

**エージェントが実装できることを、言語の質より優先した。**

[ADR-0003](0003-agent-driven-development.md) によりコードはすべてエージェントが書く。
その体制では「同一ドメインの前例があること」が実装可能性を直接左右する。
poi という**艦これで長期運用されている同一構成**の存在が、
Kotlin の言語的な優位を上回ると判断した。

## 影響

- [ADR-0004](0004-defer-tech-stack-decision.md) を置き換える（`Accepted` になった時点）
- [architecture.md](../spec/architecture.md) の骨子を埋められるようになる
- [guidelines/](../guidelines/) に TypeScript 向けの規約を書き始められる
- 対応 OS は **macOS / Windows**（Linux は同一コードで動く見込みだが対象外）
- 取り消す場合のコスト: **中**。実装着手前なら低い。
  **観測がプロキシ方式に閉じているため、ランタイムを変えても
  プロキシ部分の設計は流用できる**（案 B / C / D への乗り換え余地を残す）

## 未解決事項

- `TODO(要検証)`: **アプリ内プロキシで `/kcsapi/` のレスポンス本文を実際に取得できるか。**
  採択後の最優先事項。ここが崩れると本 ADR は成立しない
- `TODO(要検証)`: `session.setCertificateVerifyProc` で自前 CA のみを信頼させる実装が
  期待どおり動くか
- `TODO(要検証)`: poi の `lib/proxy.ts` の実装（同型の設計か）
- `TODO(未確定)`: 永続化の方式。データ量の見積もりが要求に依存するため保留
- `TODO(未確定)`: 配布方法（署名・自動更新の有無）
