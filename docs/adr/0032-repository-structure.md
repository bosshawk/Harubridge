# ADR-0032: リポジトリ構成を確定する（ADR-0026 / 0027 / 0030 を統合）

- ステータス: **Proposed**
- 日付: 2026-08-09
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 統合元: [ADR-0026](0026-injection-script-build.md)（注入スクリプトの持ち方）,
  [ADR-0027](0027-repository-layout.md)（ディレクトリ構成）,
  [ADR-0030](0030-no-named-architecture.md)（内側の分け方）。
  **3 本の調査事実・却下案の全文はそれぞれに履歴として残る。本 ADR は決定を 1 本に集約する**
  （[ADR-0015](0015-documentation-layout.md) が 6 本を統合したのと同じ形式）
- 関連: [ADR-0016](0016-tech-stack.md)（Tauri + Rust + React）,
  [ADR-0021](0021-data-persistence.md) / [ADR-0022](0022-observed-data-privacy.md)（データとフィクスチャ）,
  [ADR-0024](0024-state-sync-granularity.md)（状態同期）,
  [ADR-0029](0029-injection-ipc-transport.md)（注入スクリプトからの転送。**統合しない。別論点**）,
  [ADR-0031](0031-toolchain-management.md)（ツールチェイン）

## 背景と課題

リポジトリの構成に関する決定が 3 本の ADR に分散し、全体像を 1 箇所で読めなくなっていた。
オーナーの指示（2026-08-09）により 1 本に集約する。
あわせて「ルートのディレクトリが多い」という指摘を受け、**1 点だけ決定を変更する**
（注入スクリプトの位置。§4）。それ以外の決定は統合元から変更していない。

## 決定の原則

**フォルダとクレートの分割は、守らせたい規範を機械の管轄に移すためだけに行う。**
このプロジェクトはコードをすべて AI エージェントが書き、人間が全行を見張ることはできない
（[ADR-0003](0003-agent-driven-development.md)）。文章に書いただけのルールは破られても
気づけないため、**破った瞬間にビルドか検査が失敗する形**にする。

強制される境界は次の 3 本だけ。それ以外は規約ではなく「置き場所の目安」とする。

| # | 規範 | 強制する主体 | 破ると |
| --- | --- | --- | --- |
| 1 | コア（`crates/harubridge-core/`）は `tauri` を知らない | コンパイラ（クレート分割） | ビルドが失敗 |
| 2 | 艦これ API の語彙（`api_*`）は受信モジュールの外へ出ない | コンパイラ（private な入れ子 module） | ビルドが失敗 |
| 3 | TS: `features/` どうしが直接 import しない | リンタ（`eslint-plugin-import`） | リントが失敗 |

これに乗らない規範が 1 つある。**「通信は読むだけ」（[C-02](../spec/constraints.md)）は機械で
守らせられない。** 人間が常時レビューする前提も置かない（オーナー判断 2026-08-09）。
代わりに、**実際に注入されるテキストそのもの（生成物の `.js`）をリポジトリにコミットする**形を保ち、
監査したい者がいつでも「動くものそのもの」を読める状態にする（§4）。

## 決定（ツリー）

```
Harubridge/
├── Cargo.toml                  ワークスペース定義。members は glob を使わず完全一致で列挙
├── Cargo.lock                  コミットする
├── package.json                フロントエンドは単一パッケージ（モノレポにしない）
├── pnpm-lock.yaml              コミットする
├── index.html / vite.config.ts / tsconfig.json / eslint.config.js
├── mise.toml / rust-toolchain.toml        ADR-0031
│
├── docs/                       ADR-0015（本 ADR は触れない）
├── data/kancolle/              ADR-0020 / data/README.md（本 ADR は触れない）
│
├── crates/
│   └── harubridge-core/        Rust コア。tauri に依存しない
│       ├── Cargo.toml
│       ├── build.rs            data/kancolle/*.json を検証して OUT_DIR に吐く
│       ├── src/
│       └── tests/
│           └── fixtures/       ADR-0022 の合成フィクスチャ
│
├── src-tauri/                  Tauri アプリ（殻）。既定の名前・位置のまま
│   ├── Cargo.toml
│   ├── build.rs                tauri_build::build() のみ
│   ├── tauri.conf.json         frontendDist = "../dist"（既定のまま）
│   ├── capabilities/  icons/
│   ├── injected/
│   │   ├── kcsapi-hook.ts      注入スクリプトのソース（TypeScript。§4）
│   │   └── kcsapi-hook.js      tsc の生成物。コミットする（§6）
│   └── src/
│
├── src/                        フロントエンド（React / TypeScript）
│   ├── main.tsx
│   ├── bindings.ts             tauri-specta の生成物。コミットする（§6）
│   ├── app/                    features を合成する唯一の場所（パネル配置・通知の常駐）
│   ├── ipc/                    invoke / listen をここに集約。外に書かない
│   ├── store/                  zustand 1 ストア + ドメイン単位のスライス
│   ├── features/               docs/spec/external/ のファイル名と 1 対 1
│   │   ├── game-screen/        ← game-screen.md
│   │   ├── fleet-view/         ← fleet-view.md
│   │   └── timers/             ← timers.md
│   └── shared/                 機能をまたぐもののみ（パネルの外枠・エラー境界を含む）
│
├── .local/                     git 管理外（ADR-0021 / 0022）
├── dist/  target/  node_modules/   git 管理外
```

### §1 Rust: コアと殻の 2 クレート（境界 1）

境界は「**Tauri を知っているか**」の 1 本だけ。

| | `crates/harubridge-core/` | `src-tauri/` |
| --- | --- | --- |
| 責務 | 通信データの解釈・状態保持・永続化・任務の計数 | IPC・ウィンドウ・注入・プラグイン配線 |
| `tauri` への依存 | **持たない** | 持つ |
| 変更頻度 | ゲーム更新のたび | ほとんど変わらない |
| テスト | `cargo test -p harubridge-core`（WebView 不要） | ほぼ書かない（配線のみ） |

依存は `src-tauri` → `harubridge-core` の一方向のみ。逆流はコンパイルが通らない。
**それ以上は割らない。** 割っても新たにコンパイラが止められる違反が増えないため
（4 クレート案の却下。詳細は [ADR-0027](0027-repository-layout.md)）。

`build.rs` はコア側に置き、`cargo::rerun-if-changed=../../data/kancolle` を必ず出す
（`data/` はパッケージ外のため、これが無いと変更が検出されない）。出力は `OUT_DIR` のみ。

### §2 `src-tauri/` は既定の名前・位置のまま

公式ドキュメント・`create-tauri-app` の生成物・公開されている例がそのまま当てはまる状態を保つ。
構成が 1 段ずれるたびにエージェントが翻訳を誤る余地が生まれる（[ADR-0003](0003-agent-driven-development.md)）。

### §3 フロントエンド: 機能別ディレクトリ（境界 3）

- **ディレクトリ名を `docs/spec/external/` のファイル名に一致させる。**
  仕様との対応が目視でも機械（docs-audit）でも確認でき、1 機能の変更が 1 ディレクトリに閉じる
- 規律は 3 つ。**`features/` どうしを import しない**（共有は `shared/` へ）、
  **`shared/` は `features/` を import しない**、**`app/` だけが `features/` を合成する**。
  `eslint-plugin-import` の `import/no-restricted-paths` で機械化する（導入は承認待ち。未解決事項）
- **未仕様の機能のディレクトリを先掘りしない**
- **`shared/` の肥大を防ぐ規律**: `shared/` へ出すのは**2 人目の利用者が実際に現れたときだけ**。
  先回りで置かない。2 つの機能に同じコードが重複するのは許容する（誤った共通化より安い）。
  `shared/` にドメインロジック（計算・判定）が現れたら、それは Rust コア行きの誤配置の信号
  （[ADR-0024](0024-state-sync-granularity.md): 計算は Rust 側で済ませてから渡す）。
  機能どうしの連携は `shared/` ではなく store 経由 + `app/` での合成で行う
- ストアは 1 本にし、ドメインごとのスライスで分ける（zustand 公式推奨。
  再描画は selector で絞れるため分割と挙動は同じ）。
  **残り時間・絞り込み結果をストアに持たない**（描画時に導出）
- テストは対象ファイルの隣（`*.test.ts(x)`）。`features/<name>/` の削除で全部消える形を保つ

### §4 注入スクリプト: TypeScript で書き、型を落としただけの JS を注入する

**持ち方**（統合元 [ADR-0026](0026-injection-script-build.md) の決定「生 JS 1 枚」を
**オーナー判断 2026-08-09 で覆した**。経緯は本節末尾）:

- ソースは **TypeScript 1 ファイル**（`src-tauri/injected/kcsapi-hook.ts`）。
  ESLint（型情報つき）の対象に含める
- 変換は **`tsc` による型注釈の除去のみ**（`target: ESNext`）。
  **バンドルも構文の downlevel もしない。** ヘルパ関数や polyfill が混入せず
  （[ADR-0026](0026-injection-script-build.md) 事実 11 / 12 の懸念を回避）、
  出力はソースから型を取り除いただけのテキストになる
- 生成物 `kcsapi-hook.js` は**コミットし、CI で再生成して差分ゼロを強制する**
  （§6 の判定: 読む工程 = cargo は生成器 = tsc を実行しない。`bindings.ts` と同じ機構）
- **値の import を書かない。npm に依存しない。IIFE として自己完結させる。**
  ページの main world で動き、React も `bindings.ts` の値も存在しないため。
  型だけは `import type` で参照してよい（コンパイル時に消える）
- [C-02](../spec/constraints.md) の監査は**コミットされた `.js`（= 実際に注入されるテキスト）**に
  対して行える。ソースとの対応は CI が保証する

**経緯**: ADR-0026 は「人間がレビューするテキストと実行されるテキストの同一性」を決め手に
生 JS を採っていた。しかし**人間がこのファイルを常時レビューする前提を置かない**
（オーナー判断）。前提が消えると生 JS の優位は「生成物をコミットすれば実行されるテキストが
リポジトリに残る」ことでほぼ代替でき、残る差は記法だけになる。記法はプロジェクト標準の
TypeScript（[ADR-0016](0016-tech-stack.md)）に揃え、エージェントが最も確実に書ける形を採る。
なお型検査の強さは生 JS + `@ts-check` でも同等であり、この反転で検査能力は変わらない。
TS 化は近い一般領域（ブラウザ拡張の main world スクリプト）のデファクトとも一致する。

**位置**（[ADR-0027](0027-repository-layout.md) の決定をここだけ変更）:

- 旧決定はルート直下 `injected/` だった。**`src-tauri/injected/` に変更する。**
- 理由: ルート直下に 1 ファイルだけのディレクトリが増えることを避ける（オーナー指摘 2026-08-09）。
  埋め込む側の Rust の隣に置くのが位置としても自然
- 旧 ADR 自身がこの代替案の却下を「弱い却下」とし、取り消しコストを
  「ファイル 1 枚の移動であり低い」と記録していた。その判断を反転しただけである
- 引き換えに、`tsconfig` / ESLint の検査対象が `src-tauri/` 配下に 1 ファイルぶん伸びる。
  対象をファイル名で明示指定して重なりを最小にする

### §5 内側の構造: アーキテクチャを名乗らない

[ADR-0030](0030-no-named-architecture.md) の決定を変更なしで引き継ぐ。

- DDD / オニオン / クリーン / ヘキサゴナル / Feature-Sliced Design の語彙
  （レイヤー・ポート・アダプタ・ユースケース・エンティティ）を持ち込まない。
  却下理由の全文は ADR-0030 に残る（要旨: 書き込みが存在せず守るべき集約が無い。
  ドメインを所有していない。Rust の実プロジェクトのデファクトでもない）
- Rust コアの内側は関心事ごとの module 分割。**module 名は決めない（コードが正。ADR-0008）**
- **境界 2 の実装**: 艦これ API の `serde` 型と変換は、受信 module の**中の private な
  下位 module** に置く。Rust の可視性規則により兄弟 module から到達できず、
  コンパイルが止める（rustc 1.94.1 で実測済み。ADR-0030）
- 内部モデル（ID を持つ・保存する形）と UI へ出す形（マスタ結合済み）は分ける。
  理由は `state/*.json` がマスタ無しで復元されるため（[ADR-0024](0024-state-sync-granularity.md)）
- 外部モデルと変換は**エンドポイント別**、内部モデルは**ドメイン別**に割る
  （`api_port/port` 1 本が艦隊・入渠・資源などを同時に更新するため、軸が直交する）
- trait による抽象は実装が 2 つ以上できるまで入れない
- `crates/harubridge-core/clippy.toml` の `disallowed-methods` で
  [ADR-0025](0025-clock-handling.md) 系の規範（`Instant::now` の直接使用禁止など）を機械化できる

### §6 生成物: 「読む工程が生成器を必ず実行するか」で決める

- **必ず実行する** → コミットしない（例: `build.rs` の出力。`OUT_DIR` のみ。Cargo の明文規則）
- **実行しないことがありうる** → **コミットし、CI で再生成して差分ゼロを強制する**

| 生成物 | 置き場所 | コミット |
| --- | --- | --- |
| `tauri-specta` の TypeScript 型 | `src/bindings.ts` | **する**（生成はアプリの debug 実行時のため、しないとクローン直後にフロントの検査が通らない） |
| 注入スクリプトの JS（`tsc` の出力） | `src-tauri/injected/kcsapi-hook.js` | **する**（読む工程 = cargo が tsc を実行しないため。§4） |
| `data/kancolle/*.json` の検証・埋め込み結果 | `OUT_DIR` | しない |
| テストフィクスチャ | `crates/harubridge-core/tests/fixtures/` | **する**（[ADR-0022](0022-observed-data-privacy.md)） |

## 検討した選択肢

統合元 3 本で比較済み。**却下案の全文と調査事実は各 ADR に履歴として残る。** 要点のみ:

| 却下した案 | 却下理由の要旨 | 全文 |
| --- | --- | --- |
| 単一クレート（`src-tauri/` のみ） | 「コアが Tauri を知らない」が規約のままになり、違反してもビルドが通る | [0027](0027-repository-layout.md) |
| 観測/解釈/永続化/IPC の 4 クレート | コンパイラが新たに止められる違反が増えない | [0027](0027-repository-layout.md) |
| `src-tauri` を `crates/` 下へ移す（GitButler 方式） | 公式の例と 1 段ずれ、翻訳コストだけ増える | [0027](0027-repository-layout.md) |
| フロントの種類別ディレクトリ（`components/` 等） | 1 機能が 4 ディレクトリに散る（実測: Clash Verge Rev） | [0027](0027-repository-layout.md) / [0030](0030-no-named-architecture.md) |
| 注入スクリプトを生 JS 1 枚で持つ | **旧決定（ADR-0026 案 A）。本 ADR で反転** —— 人間が常時レビューする前提を置かないため、同一性の優位が消える | 本 ADR §4 |
| 注入スクリプトを TS + バンドラで作る | 分割と npm 依存は満たしてはいけない要件。バンドラの設定が監査対象に増える。`tsc` の型除去だけで足りる | [0026](0026-injection-script-build.md) |
| 注入スクリプトをルート直下に置く | **旧決定。本 ADR で反転**（ルートの項目を減らす） | 本 ADR §4 |
| DDD / オニオン / ヘキサゴナル / FSD | 守るべき書き込み・所有するドメインが無く、層が空になる | [0030](0030-no-named-architecture.md) |
| `bindings.ts` を .gitignore する | クローン直後にフロントの型検査・ビルドが通らなくなる | [0027](0027-repository-layout.md) |

## 決め手

**機械が止められる違反の数だけ境界を作り、それ以外の自由を残した。**
分割の単位はコンパイラ 2 本・リンタ 1 本の計 3 本の規範と 1 対 1 に対応しており、
それに対応しない分割（4 クレート・層の命名・種類別ディレクトリ）はすべて却下されている。

## 影響

- 実装への影響:
  - ルートに `Cargo.toml`（ワークスペース）。`Cargo.lock` と `target/` はルートに置かれる
  - `build.rs` は 2 枚（コア: データ検証 / 殻: `tauri_build`）
  - CI に 2 本のジョブが要る: `bindings.ts` の再生成差分検査、フィクスチャのバイト一致検査
  - `cargo test -p harubridge-core` が WebView 無しで完結する
- ドキュメントへの影響:
  - [ADR-0026](0026-injection-script-build.md) / [ADR-0027](0027-repository-layout.md) /
    [ADR-0030](0030-no-named-architecture.md) を `Superseded by ADR-0032` にする
  - [architecture.md](../spec/architecture.md) の「コアは `tauri` に依存しない」の 1 行追記は
    0027 からの持ち越し提案のまま（**人間の承認が必要**）
- 取り消す場合のコスト: 統合元の記録と同じ（クレート統合: 低 / ディレクトリ移動: 低〜中 /
  `bindings.ts` の反転: 低）

## 未解決事項

統合元から引き継ぐもの（解消したものは除いた）:

- `TODO(承認待ち)`: **`eslint-plugin-import` の導入**（[ADR-0018](0018-dependencies.md) 系列の承認事項）。
  入れるまで境界 3 は規約に留まり、違反しても検出されない
- `TODO(未確定)`: CI の構成。`bindings.ts` の差分検査もフィクスチャ検査も CI 前提であり、
  CI が無い間は手編集を止める手段が無い
- `TODO(未確定)`: フィクスチャ生成器の実装形態（[ADR-0022](0022-observed-data-privacy.md) のまま）
- `TODO(未確定)`: `.taurignore` を置くか。実際に困るまで置かない
- `TODO(要検証)`: ワークスペース構成での `tauri build` を実際に通していない。最初のビルドで確認する
- 注入スクリプトから Rust への転送経路は [ADR-0029](0029-injection-ipc-transport.md)（別論点、Proposed）
