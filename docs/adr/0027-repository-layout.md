# ADR-0027: リポジトリのディレクトリ構成をコア／殻の 2 クレート＋機能別フロントエンドにする

- ステータス: **Proposed**
- 日付: 2026-08-03
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0016](0016-tech-stack.md)（Tauri + Rust + React）,
  [ADR-0018](0018-dependencies.md)（`tauri-specta`）, [ADR-0019](0019-linter.md),
  [ADR-0021](0021-data-persistence.md)（`state/` `events/` `replays/`）,
  [ADR-0022](0022-observed-data-privacy.md)（`.local/` / フィクスチャ生成器）,
  [ADR-0026](0026-injection-script-build.md)（注入スクリプトのビルド方式。**本 ADR は置き場所のみを決め、
  中身の作り方はそちらに従う**。両方とも Proposed であり、ADR-0026 が覆れば本 ADR の §5 も見直す）,
  [docs/spec/architecture.md](../spec/architecture.md), [data/README.md](../../data/README.md)
- **対象外**: `docs/` 以下の構成。[ADR-0015](0015-documentation-layout.md) で決着済みであり、本 ADR は触れない

## 背景と課題

コードがまだ 1 行も無い。一方で、置き場所を決めていないと動けないものが既に 4 つある。

1. **Rust をどう割るか。** [ADR-0016](0016-tech-stack.md) は Tauri を選び、
   [architecture.md](../spec/architecture.md) は「注入スクリプト / Rust コア / UI」の 3 つに
   責務を分けている。しかし「Rust コア」が Tauri アプリと同じクレートなのかは決まっていない
2. **注入スクリプトをどこに置くか。** Rust からもフロントのビルドからも参照されうる
3. **生成物をどこに置き、コミットするか。** [ADR-0018](0018-dependencies.md) の `tauri-specta` は
   TypeScript を吐き、[architecture.md](../spec/architecture.md) は `build.rs` が
   `data/kancolle/*.json` を検証・埋め込みすると定めている
4. **テストをどこに置くか。** [ADR-0022](0022-observed-data-privacy.md) は
   フィクスチャを `tests/fixtures/` に置き、CI で生成器の出力とのバイト一致を強制すると決めている。
   その `tests/` がどのクレートのものかが未定である

**既に決まっていて動かせないもの**（本 ADR はこれらに合わせる）:

| 場所 | 決定 |
| --- | --- |
| `docs/` | [ADR-0015](0015-documentation-layout.md) |
| `data/`（トップレベル） | [data/README.md](../../data/README.md) / [ADR-0020](0020-kancolle-reference.md) |
| `.local/`（git 管理外、debug のデータ置き場） | [ADR-0022](0022-observed-data-privacy.md) |
| `state/` `events/` `replays/`（上記ルートの下） | [ADR-0021](0021-data-persistence.md) |

### 調査した事実

いずれも 2026-08-03 に一次ソースを当たって確認した。

**Tauri v2 の既定構成**（[公式 Project Structure](https://v2.tauri.app/start/project-structure/)）

```
.
├── package.json
├── index.html
├── src/
│   ├── main.js
├── src-tauri/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── src/
│   │   ├── main.rs
│   │   └── lib.rs
│   ├── icons/
│   └── capabilities/
│       └── default.json
```

同ページは「Rust のコードだけで作業したいなら、`src-tauri/` をトップレベルの
プロジェクトとして使うか、**あるいは Rust ワークスペースのメンバーとして使う**」と明記している。
**ワークスペース化は公式に想定された使い方である。**

**`tauri.conf.json` の位置がすべてを決める**（[公式 Configuration Files](https://v2.tauri.app/develop/configuration-files/) /
[Config リファレンス](https://v2.tauri.app/reference/config/)）

- `frontendDist` は「設定ファイルからの相対パス」として解決される（既定値は `../dist`）
- Tauri のメンテナ FabianLars は
  [tauri-apps/discussions#13941](https://github.com/orgs/tauri-apps/discussions/13941) で
  「`src-tauri` フォルダはその名前である必要はない。**`tauri.conf.json` が
  Tauri アプリの `Cargo.toml` の隣にあればよい**」と述べている。
  同スレッドで「`target` ディレクトリの位置が問題になることはめったにない。
  Cargo ワークスペースにするならワークスペースルートに置かれる」とも述べている

**ワークスペースにしたときの既知の落とし穴**（すべて実際に報告されたもの）

| # | 内容 | 状態 |
| --- | --- | --- |
| [#2614](https://github.com/tauri-apps/tauri/issues/2614) | CLI がワークスペースの `target` 出力先を誤認する。メンバーを glob で書くと CLI の完全一致判定から漏れる。回避策は `CARGO_TARGET_DIR` の明示 | **Closed** |
| [#5865](https://github.com/tauri-apps/tauri/issues/5865) | トップレベルのワークスペースを足すと**モバイルの**バンドラが壊れる（dylib のパスがハードコードされている） | **Closed**。かつ**モバイル限定**で、本プロジェクトは macOS / Windows のみ |
| [#6252](https://github.com/tauri-apps/tauri/issues/6252) | ワークスペース継承（`authors = { workspace = true }` 等）でビルドが落ちる | **Closed as not planned**。報告は **Tauri 1.2.4** |

**ワークスペース継承は v2 では動いている。** GitButler（Tauri v2 / `tauri-build` 2.6.1）の
[`crates/gitbutler-tauri/Cargo.toml`](https://github.com/gitbutlerapp/gitbutler/blob/master/crates/gitbutler-tauri/Cargo.toml)
は `edition.workspace = true` / `authors.workspace = true` を使っている。#6252 は v1 の事象である。

**`tauri dev` はワークスペースのメンバーを監視する**（[公式 Develop](https://v2.tauri.app/develop/)）

> `tauri dev` watches your `src-tauri` folder and its dependent crates in the workspace for changes,
> so your application is automatically rebuilt and restarted whenever you modify them.

**依存クレートに切り出しても自動再ビルドは効く。** 監視を切るなら `--no-watch`、
除外するなら `.taurignore`（`.gitignore` と同じ書式。`src-tauri` かワークスペースルートに置く）。
[#4617](https://github.com/tauri-apps/tauri/issues/4617) で要望され、実装済みである。

**コアロジックを別クレートに切り出している実プロジェクト**

- **GitButler**（Tauri v2）—— ルートのワークスペースに 30 本超の `but-*` / `gitbutler-*` クレートがあり、
  Tauri アプリは `crates/gitbutler-tauri`（**`src-tauri` という名前ではない**）。
  `tauri.conf.json` はその `Cargo.toml` の隣にある。フロントエンドは `apps/desktop`
- **Spacedrive** —— ルートの [`Cargo.toml`](https://github.com/spacedriveapp/spacedrive/blob/main/Cargo.toml) が
  `core`, `crates/*`, `apps/tauri/sd-tauri-core`, `apps/tauri/src-tauri`, `xtask` をメンバーに持つ。
  **Tauri アプリのクレートと、その外にあるコアクレートが同居する形**

**`tauri-specta` v2 の型出力**（[upstream の example](https://github.com/specta-rs/tauri-specta/blob/main/examples/app/src-tauri/src/main.rs)）

```rust
#[cfg(debug_assertions)]
{
    builder
        .export(Typescript::default(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");
}
```

- 出力は **`main()` の中、`debug_assertions` の下で、アプリの実行時**に行われる。
  ビルドスクリプトでも `cargo test` でもない
- パスは Tauri アプリのクレートからの相対（`../src/bindings.ts` = プロジェクトルートの `src/`）
- **upstream 自身が生成物 `examples/app/src/bindings.ts` をコミットしている。**
  リポジトリの `.gitignore` に除外は無い
- 生成ファイルの先頭には `This file has been generated by Specta. DO NOT EDIT.` が入る

→ **重要な帰結**: 生成をコミットしない場合、クローン直後のリポジトリには `bindings.ts` が存在せず、
**アプリを一度 debug で起動するまでフロントエンドの型検査・リント・ビルドが通らない。**

**Cargo の規約**（[Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) /
[Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)）

- 「Scripts should not modify any files outside of that directory（= `OUT_DIR`）」
- `OUT_DIR` は `target/` 配下であり、コミット対象になりえない
- `rerun-if-changed` を 1 つも出さない場合、Cargo は保守的に
  「**パッケージ内の**いずれかのファイルが変わったら再実行」する。
  → **`data/` はパッケージ外なので、`cargo::rerun-if-changed` の明示が必須**（省略できない）。
  パスがディレクトリなら配下全体を走査する
- ビルドスクリプトの**カレントディレクトリはそのパッケージのルート**である
- 結合テストは `tests/`。データファイルは `tests/fixtures/` に置くのが通例
  （[ADR-0022](0022-observed-data-privacy.md) が既に確認済み）

## 決定

**リポジトリルートを Cargo ワークスペースにし、Rust を 2 クレートに割る。**
Tauri アプリは `src-tauri/` に**既定の名前と位置のまま**置き、コアを `crates/harubridge-core/` に出す。
フロントエンドは `docs/spec/external/` と 1 対 1 に対応する機能別ディレクトリにする。

```
Harubridge/
├── Cargo.toml                  ワークスペース定義（members / workspace.dependencies）
├── Cargo.lock                  ワークスペースルートに 1 本。コミットする
├── package.json                フロントエンドの単一パッケージ
├── index.html                  Vite のエントリ
├── vite.config.ts
├── tsconfig.json
├── eslint.config.js            ADR-0019
│
├── docs/                       ADR-0015（本 ADR は触れない）
├── data/                       data/README.md（本 ADR は触れない）
│   └── kancolle/
│
├── injected/                   注入スクリプト（生 JS 1 ファイル / ADR-0026）
│   └── kcsapi-hook.js          Rust が埋め込み、tsc と ESLint が検査する
│
├── crates/
│   └── harubridge-core/        Rust コア。**tauri に依存しない**
│       ├── Cargo.toml
│       ├── build.rs            data/kancolle/*.json を検証して OUT_DIR に吐く
│       ├── src/
│       └── tests/
│           └── fixtures/       ADR-0022 の合成フィクスチャ
│
├── src-tauri/                  Tauri アプリ（殻）。既定の位置・既定の名前
│   ├── Cargo.toml
│   ├── build.rs                tauri_build::build()
│   ├── tauri.conf.json         frontendDist = "../dist"（既定のまま）
│   ├── capabilities/
│   ├── icons/
│   └── src/
│       ├── main.rs
│       └── lib.rs
│
├── src/                        フロントエンド（React / TypeScript）
│   ├── main.tsx
│   ├── bindings.ts             tauri-specta の生成物。**コミットする**
│   ├── ipc/                    listen 登録と invoke の初期 pull（ADR-0030）
│   ├── store/                  ドメイン単位のストア（ADR-0024 / ADR-0030）
│   ├── features/
│   │   ├── game-screen/        ← docs/spec/external/game-screen.md
│   │   ├── fleet-view/         ← docs/spec/external/fleet-view.md
│   │   └── timers/             ← docs/spec/external/timers.md
│   └── shared/                 機能をまたぐもののみ
│
├── .local/                     ADR-0022 / ADR-0021（git 管理外）
├── dist/                       Vite の出力（git 管理外）
└── target/                     ワークスペースルートに移る（git 管理外）
```

### 1. Rust: コア（`crates/harubridge-core/`）と殻（`src-tauri/`）の 2 クレート

**境界は 1 本だけ引く。「Tauri を知っているか」である。**

| | `crates/harubridge-core/` | `src-tauri/` |
| --- | --- | --- |
| 責務 | 受信データの正規化・パース・状態保持・永続化・任務の計数 | IPC コマンドとイベント、ウィンドウ、注入、プラグイン配線 |
| `tauri` への依存 | **持たない** | 持つ |
| 変更の頻度 | ゲーム更新のたび | ほとんど変わらない |
| テスト | `cargo test` で GUI 無しに回る | ほぼ書かない（配線のみ） |

依存の向きは `src-tauri` → `harubridge-core` の一方向のみ。逆流は**コンパイルが通らない**。

これにより、[architecture.md](../spec/architecture.md) の
「UI はゲームの通信を直接見ない」「非公開仕様への依存はパース層に閉じ込める」という規範のうち、
**「コアが IPC・UI・Tauri の都合を知らない」部分が規約ではなく型検査になる。**

**内側をさらに割らない。** 観測 / 解釈 / 永続化 / IPC のうち、
Rust のクレート境界が今日「強制」できるのは上の 1 本だけである。
観測は JS（`injected/`）でありそもそも Rust ではない。解釈と永続化はどちらもコアの内側にあり、
両者を分けても**コンパイラが止められる違反が増えない**。これらは module 境界で足りる。

### 2. `src-tauri/` を既定の位置・既定の名前のまま置く

GitButler のように `crates/harubridge-tauri/` へ動かすことは可能である
（`tauri.conf.json` が `Cargo.toml` の隣にあればよい）。**それでも動かさない。**

- `frontendDist = "../dist"`、`beforeDevCommand`、アイコン生成、`capabilities/` の探索、
  `create-tauri-app` の生成物、公式ドキュメントのすべての例が**そのまま当てはまる**
- [ADR-0003](0003-agent-driven-development.md) の体制では、
  公開されている例と手元の構成がずれるたびに、エージェントが翻訳を誤る余地が生まれる
- 上記 #2614（`target` 出力先の誤認）は**メンバーを glob で書いたときに顕在化**した。
  したがって `members` は `["src-tauri", "crates/harubridge-core"]` と**完全一致で列挙する**

### 3. `build.rs` は**コア側**に置く

`data/kancolle/*.json` を使うのはコアの解釈層である。したがって
`crates/harubridge-core/build.rs` が読み、`OUT_DIR` にコードを吐く。
`src-tauri/build.rs` は `tauri_build::build()` のためだけに残る。**`build.rs` は 2 つになる。**

コア側の `build.rs` が守ること:

- カレントディレクトリはパッケージルートなので、`CARGO_MANIFEST_DIR` を起点にして
  `../../data/kancolle` を読む
- **`cargo::rerun-if-changed=../../data/kancolle` を必ず出す。**
  `data/` はパッケージ外にあるため、これが無いと Cargo の既定は変更を検出しない
- 出力は `OUT_DIR` のみ。Cargo が「`OUT_DIR` の外のファイルを変更してはならない」と定めている

### 4. フロントエンド: 機能別（`src/features/<外部仕様と同名>/`）

**ディレクトリ名を `docs/spec/external/` のファイル名に一致させる。**

- [CLAUDE.md](../../CLAUDE.md) §0 は「ドキュメントとの対応関係を通常以上に重視する」と定め、
  ユーザーに見える振る舞いは `docs/spec/external/<機能グループ>.md` に書くと決めている。
  **名前を一致させれば、仕様書と実装の対応が目視で確認でき、
  [docs-audit](../../.claude/skills/docs-audit/SKILL.md) が機械的に突き合わせられる**
- 種類別（`components/` `hooks/` `stores/`）にすると、
  1 つの外部仕様の変更が毎回 3〜4 ディレクトリに散る。対応関係は人の記憶にしか残らない

規律を 2 つ置く。

1. **`features/` どうしを import しない。** 共有したくなったら `src/shared/` へ出す
2. **`shared/` は `features/` を import しない**（依存の向きを一方向に保つ）

これは Rust 側のクレート境界と同じ趣旨だが、**TypeScript には強制手段が同梱されていない。**
`eslint-plugin-import` の `import/no-restricted-paths` で zone を定義すれば機械化できる
（[ADR-0030](0030-no-named-architecture.md)）。**プラグインの追加には承認が要る**（[未解決事項](#未解決事項)）。

### 5. 注入スクリプト: トップレベルの `injected/`

[ADR-0026](0026-injection-script-build.md)（Proposed）は
**生 JS 1 ファイル・バンドルもトランスパイルもしない・`@ts-check` と ESLint で検査する**と決めた。
**したがって生成物は存在せず、置き場所を決める問題は「1 枚のソースをどこに置くか」に縮む。**

`src/` にも `src-tauri/` にも置かず、**独立させる。**

- **参照者が両側にいる。** Rust はこれを文字列として埋め込み、
  `tsconfig`（`allowJs` / `checkJs`）と ESLint はこれを検査する
  （[ADR-0026](0026-injection-script-build.md) の「実装への影響」）。
  **どちらか一方の配下に置くと、もう一方の設定がその配下へ手を伸ばす**
- **フロントアプリの一部ではない。** ゲームページの world で動き、React も
  `bindings.ts` の値も使えない（[ADR-0026](0026-injection-script-build.md) 事実 4〜6）。
  `src/` に置くと Vite のアプリツリーに入り、
  **アプリのコードを import しても「ビルドは通ってしまう」**。
  ADR-0026 の「import 文を書かない」という前提を破りやすくする
- **この 1 ファイルだけが [C-02](../spec/constraints.md) を担保している。**
  ADR-0026 の決め手は「人間がレビューするテキストと、ページで実行されるテキストが同一であること」だった。
  監査対象がトップレベルに 1 つあるほうが、`src-tauri/src/` の中に紛れるより見つけやすい
- これは既に決まっている **`data/` をトップレベルに置いた理由と同型**である
  （[data/README.md](../../data/README.md):「`docs/` でも `src-tauri/` でもない」）

**ADR-0026 の結論に依存する部分**は次の 1 点だけである。

> **`injected/` の下に生成物が置かれることは無い**、という前提。
> ADR-0026 が覆って案 C（バンドラ）に移った場合、
> `injected/` は「ソース＋生成物」を持つことになり、
> そのとき生成物の扱いは後述 §6 の判定（生成器を必ず実行する工程しか読まないなら
> コミットしない）に従う。**`injected/` という位置そのものは、どちらでも動かない。**

なお ADR-0026 の結論の下では `src-tauri/injected/` でも成立し、差は小さい。
**取り消しコストはファイル 1 枚の移動であり、低い。**

### 6. 生成物: 「読む工程が生成器を必ず実行するか」で決める

**判定基準はこれ 1 本にする。**

- **必ず実行する** → コミットしない
- **実行しないことがありうる** → **コミットし、CI で再生成して差分ゼロを強制する**

| 生成物 | 生成者 | 置き場所 | コミット |
| --- | --- | --- | --- |
| `tauri-specta` の TypeScript 型 | `src-tauri`（debug 実行時） | `src/bindings.ts` | **する** |
| `data/kancolle/*.json` の検証・埋め込み結果 | `crates/harubridge-core/build.rs` | `OUT_DIR`（`target/` 配下） | しない |
| テストフィクスチャ | 生成器（形態は ADR-0022 で未確定） | `crates/harubridge-core/tests/fixtures/` | **する**（[ADR-0022](0022-observed-data-privacy.md) の決定） |

**`bindings.ts` をコミットする理由**は、`tauri-specta` の出力が
**アプリの debug 実行時**に行われることにある（上記の調査事実）。
コミットしないと、クローン直後は `tsc` も ESLint も `vite build` も通らず、
**フロントエンドの CI が Rust のビルドとアプリの起動に依存する。**
upstream 自身も生成物をコミットしている。

代償は「生成物が差分に出る」ことである。これは
**[ADR-0022](0022-observed-data-privacy.md) がフィクスチャに対して既に採った機構
（生成器を再実行してバイト一致を CI で強制する）をそのまま適用して打ち消す。**
同じ問題に同じ道具を使い、規律を 1 種類に保つ。

**`OUT_DIR` に落とすものは選択の余地が無い。** Cargo が明文で禁じている。

### 7. テスト

| 対象 | 置き場所 | 根拠 |
| --- | --- | --- |
| コアの単体テスト | ソースと同じファイルの `#[cfg(test)] mod tests` | Rust の慣習 |
| コアの結合テスト | `crates/harubridge-core/tests/` | [Cargo Package Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html) |
| フィクスチャ | `crates/harubridge-core/tests/fixtures/` | [ADR-0022](0022-observed-data-privacy.md) |
| 実測データを読む任意実行テスト | 同上（データは `.local/` から読む） | [ADR-0022](0022-observed-data-privacy.md)（CI では走らせない） |
| `src-tauri/` のテスト | 原則として置かない | 配線しか無いため |
| フロントエンド | 対象ファイルの隣（`*.test.ts` / `*.test.tsx`） | 機能ディレクトリを自己完結させ、`features/<name>/` の削除で全部消えるようにする |

**コアが `tauri` に依存しないことは、そのままテストのしやすさになる。**
パース層のテストは WebView もウィンドウも要らず、`cargo test -p harubridge-core` で完結する。

## 検討した選択肢

### 論点 1: Rust をどう割るか

#### 案 A1: `crates/harubridge-core/` ＋ `src-tauri/` の 2 クレート（採用）

- 概要: 上記のとおり。境界は「Tauri を知っているか」1 本
- 利点:
  - 層の逆流をコンパイラが止める。[ADR-0016](0016-tech-stack.md) の決め手
    （「人間の常時レビューが無い体制ではコンパイラが最後の防波堤」）と同じ論法が
    ディレクトリ構成にも効く
  - パース層のテストが GUI 依存から切り離される（[ADR-0022](0022-observed-data-privacy.md) の CI が軽くなる）
  - 変更頻度が違うもの（ゲーム更新に追随する解釈 / ほぼ変わらない殻）が分かれる
  - `tauri dev` は依存クレートも監視するため、開発体験は落ちない（公式ドキュメントで確認済み）
- 欠点:
  - `Cargo.toml` が 2 枚、`build.rs` が 2 枚になる
  - ワークスペース化に伴う CLI の既知の粗さ（#2614）を踏む可能性が残る。
    メンバーを完全一致で列挙することで回避する

#### 案 A2: Tauri の既定のまま `src-tauri/` 単一クレート

- 概要: `create-tauri-app` の出力をそのまま使い、内部は module で分ける
- 利点:
  - 構成が最小。公式ドキュメントと完全に一致し、ワークスペース由来の問題が一切起きない
  - コードが 1 行も無い段階で境界を引かずに済む（後から切り出せる）
- 却下理由: 「コアが Tauri を知らない」という
  [architecture.md](../spec/architecture.md) の中心的な規範が、
  **module 境界では規約のままになり、違反してもビルドが通る**ため。

#### 案 A3: 観測 / 解釈 / 永続化 / IPC を 4 クレートに分ける

- 概要: 責務ごとにクレートを立て、依存の向きを全境界で強制する
- 却下理由: 観測は JS であり Rust のクレートにならず、解釈と永続化を分けても
  **コンパイラが新たに止められる違反が無い**ため。境界の数だけ管理コストが増える。

#### 案 A4: GitButler 方式（`src-tauri` を `crates/` の下に移し、名前も変える）

- 概要: `crates/harubridge-tauri/` に `tauri.conf.json` を同居させる。
  Tauri v2 で実際に動いている構成である（実例で確認済み）
- 却下理由: Tauri の既定パス（`frontendDist = "../dist"` 等）と公開されている例の前提が
  1 段ずれ、[ADR-0003](0003-agent-driven-development.md) の体制でその翻訳コストを払う理由が無いため。

### 論点 2: フロントエンドのディレクトリ規約

#### 案 B1: 機能別。名前を `docs/spec/external/` に合わせる（採用）

- 利点: 仕様書と実装の対応が目視・機械の両方で確認できる。
  1 機能の変更が 1 ディレクトリに閉じ、削除も 1 ディレクトリで済む
- 欠点: 共有物の置き場所（`shared/`）の判断が都度必要になる。
  機能をまたぐ表示（例: どこにでも出る通知）の帰属が曖昧になりうる

#### 案 B2: 種類別（`components/` `hooks/` `stores/` …）

- 却下理由: 外部仕様 1 本の変更が毎回 3〜4 ディレクトリに散り、
  [CLAUDE.md](../../CLAUDE.md) §0 が求める「ドキュメントとの対応関係」が
  ディレクトリ構造から読み取れなくなるため。

### 論点 3: 注入スクリプトの置き場所

#### 案 C1: トップレベルの `injected/`（採用）

- 利点: Rust の埋め込みと `tsconfig` / ESLint の検査という**両側からの参照**に対して中立。
  [C-02](../spec/constraints.md) を担保する唯一のファイルが最上位で目に入る。
  [ADR-0026](0026-injection-script-build.md) がどちらに転んでも位置が動かない
- 欠点: トップレベルのディレクトリが 1 つ増える。
  ADR-0026 の結論の下ではファイル 1 枚だけであり、大げさに見える

#### 案 C2: `src-tauri/` の下（例: `src-tauri/injected/`）

- 概要: Rust が埋め込む前提では最も近い場所にある。
  **[ADR-0026](0026-injection-script-build.md) の結論（生成物なし）の下では、これも成立する**
- 却下理由: `tsconfig` の `include` と ESLint の対象が Rust クレートの配下へ伸び、
  Rust 側のパッケージ境界（Cargo の「パッケージ内の変更」判定、rust-analyzer の走査範囲）と
  重なるため。**差は小さく、これは弱い却下である。取り消しコストは低い。**

#### 案 C3: `src/` の下（フロントエンドの一部として扱う）

- 却下理由: 実行される world が違い、React も `bindings.ts` も使えないにもかかわらず、
  **アプリのコードを import してもビルドが通ってしまう**ため。
  [architecture.md](../spec/architecture.md) の「注入スクリプトは判断を持たない」を守りにくくする。

### 論点 4: 生成物をコミットするか

#### 案 D1: `bindings.ts` はコミットし、CI で差分ゼロを強制する（採用）

- 利点: クローン直後にフロントエンドの型検査・リント・ビルドが通る。
  レビュー時に IPC の型変更が差分として見える。upstream の運用と一致する。
  検査の機構が [ADR-0022](0022-observed-data-privacy.md) と同じで、規律が 1 種類に収まる
- 欠点: 生成物が差分に混ざる。CI が未整備な間は「差分ゼロ」を強制できない

#### 案 D2: `bindings.ts` を `.gitignore` し、ビルド前に生成する

- 却下理由: `tauri-specta` v2 の出力は**アプリの debug 実行時**に行われるため、
  ビルド前生成にするには生成専用の bin かテストへ出す作りが別途要る。
  それを用意しても、フロントエンドの CI が Rust ツールチェーンに依存する点は残るため。

#### 案 D3: `build.rs` の出力を `OUT_DIR` 以外（例: `src/generated/`）に吐いてコミットする

- 却下理由: Cargo が「ビルドスクリプトは `OUT_DIR` の外のファイルを変更してはならない」と
  明文で定めているため。加えて、この生成物を読むのは Rust のビルドだけであり、
  読む工程が必ず生成器を実行する。

## 決め手

**Tauri 既定構成からの逸脱を最小限（ワークスペース化とコアの切り出しのみ）に抑えることと引き換えに、
「層の逆流をコンパイラが止める」ことを取った。**

境界をクレートに昇格させるのは、**そこに機械的に強制したい規範があるとき**に限る。
それ以外は module とディレクトリで足りる。
生成物の扱いも同じ形で決めた —— **人が守る規約ではなく、CI が落ちる形にできるかどうか**で分岐している。

## 影響

- 実装への影響:
  - ルートに `Cargo.toml`（ワークスペース）が生まれ、`Cargo.lock` と `target/` が
    `src-tauri/` からルートへ移る。既存の [.gitignore](../../.gitignore) の `target/` は両方に効く
  - `members` は **glob を使わず完全一致で列挙する**（#2614 の回避）
  - `build.rs` が 2 つになる。コア側は `cargo::rerun-if-changed=../../data/kancolle` を必ず出す
  - CI ジョブが 2 本増える: `bindings.ts` の再生成差分検査、フィクスチャの
    バイト一致検査（後者は [ADR-0022](0022-observed-data-privacy.md) の既定）
  - `cargo test -p harubridge-core` が WebView 無しで完結する
- ドキュメントへの影響（**人間の承認が必要**）:
  - [architecture.md](../spec/architecture.md) の 3 分割（注入スクリプト / Rust コア / UI）は
    本 ADR と矛盾しない。**クレート分割は「Rust コア」の内側の話であり、
    [ADR-0008](0008-code-as-source-of-truth.md) によりそれ自体は文書化しない**
  - ただし「**コアは `tauri` に依存しない**」は機能追加時に必ず従う構造上の制約であり、
    コードからは読み取りにくい（`Cargo.toml` を見ないと分からない）。
    [architecture.md](../spec/architecture.md)「機能追加時の構造上の制約」に 1 行足すことを提案する
  - `docs/guidelines/` は本 ADR では起こさない。
    [ADR-0022](0022-observed-data-privacy.md) が予定しているテスト規約の中で参照すれば足りる
- 取り消す場合のコスト:
  - 2 クレートを 1 つに戻す: **低**（機械的な統合）
  - `src-tauri/` を移動する: **中**（`tauri.conf.json` の相対パスと CI の参照が動く）
  - `injected/` の移動: **低**（ファイルの移動のみ）
  - `bindings.ts` のコミット可否の反転: **低**

## 未解決事項

- **[ADR-0026](0026-injection-script-build.md) に依存**: `injected/` の下に生成物が置かれないこと。
  ADR-0026 も Proposed であり、覆って案 C（バンドラ）に移った場合は
  `injected/` がソースと生成物を持つ形になる（位置そのものは動かない）。
  本 ADR が決めたのは**置き場所が `injected/` であること**だけで、中身の作り方は ADR-0026 に従う
- ~~`TODO(未確定)`: `features/` 間の相互 import を機械的に禁じる手段。~~
  **→ 解消。** `eslint-plugin-import` の `import/no-restricted-paths` で
  zone を定義すれば強制できる（[ADR-0030](0030-no-named-architecture.md)）。
  **プラグインの追加は [ADR-0018](0018-dependencies.md) 系列の承認事項として残る。**
  入れるまでは規約に留まり、違反しても検出されない
- `TODO(未確定)`: フィクスチャ生成器の実装形態（`cargo test` 内か独立した bin か）。
  [ADR-0022](0022-observed-data-privacy.md) の未解決事項のまま。置き場所だけが本 ADR で決まった
- `TODO(未確定)`: CI の構成そのもの。[ADR-0022](0022-observed-data-privacy.md) が
  「CI 自体が未整備」と記録している。**本 ADR の生成物の扱いは CI の差分検査を前提にしている**ため、
  CI が無い間は `bindings.ts` の手編集を止める手段が無い
- `TODO(未確定)`: `.taurignore` を置くか。`injected/` や `data/` の変更で
  `tauri dev` が再ビルドしすぎる場合に必要になるが、実際に困るまで置かない
- `TODO(要検証)`: ワークスペース化した状態での `tauri build`（バンドル出力先）を
  実際に通していない。**最初のビルドを通した時点で確認すること**（Windows は [Issue #6](https://github.com/bosshawk/Harubridge/issues/6)）。
  ただし **#2614 の修正時期は特定できた** —— tauri-cli **1.0.0-rc.0（commit `8d630bc8`、2021-09-23）**で修正され、
  さらに **1.2.0 で `cargo metadata` ベースに置き換わっている**。
  現行は `metadata.target_directory` を使うため出力先を誤認しない（2026-08-08 に tauri-cli 2.11.4 のソースで確認）
- `TODO(未確定)`: 配布用のアイコンと `capabilities/` は Tauri 既定のままにしてある。
  配布方法（[ADR-0018](0018-dependencies.md) の未解決事項）が決まるまで触らない
