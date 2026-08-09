# ADR-0031: 開発ツールチェインを mise に統一し、Rust の版は `rust-toolchain.toml` を正として mise に読ませる

- ステータス: **Accepted**
- 日付: 2026-08-09
- 決定者: プロジェクトオーナー
- 関連: [ADR-0016](0016-tech-stack.md)（Rust + TypeScript）,
  [ADR-0018](0018-dependencies.md)（依存の選定方針）,
  [ADR-0027](archive/0027-repository-layout.md)（リポジトリ構成。ルートに 2 ファイル増える）,
  [ADR-0003](0003-agent-driven-development.md), [ADR-0005](0005-publish-as-oss.md),
  調査: [Issue #7](https://github.com/bosshawk/Harubridge/issues/7)（本 ADR の起票により閉じる）

## 背景と課題

[ADR-0016](0016-tech-stack.md) が Rust と TypeScript を選び、[ADR-0018](0018-dependencies.md) が
その内側のライブラリを確定させたが、**それらを動かす道具の版を誰が固定するのかは未決だった。**

具体的に管理者が不在だったものが 3 つある。

1. **Node** —— `.nvmrc` は何も実行しないただのテキストであり、読んで Node を入れる主体が別に要る
2. **pnpm** —— `package.json` の `packageManager` フィールドが効くのは pnpm が入った後。最初の 1 本は別の仕事
3. **Rust** —— rustup という公式の管理者は存在するが、**オーナー環境ではその rustup が壊れていた。**
   Homebrew で入れた rustup を削除した結果、`~/.cargo/bin` の 14 エントリすべてが
   切れたシンボリックリンクになり（12 本が `~/.cargo/bin/rustup` を、それが消えた
   `rustup-init` を指す連鎖）、`rustc` も `cargo` も PATH から消えていた。
   一方 `~/.rustup/toolchains/`（1.2G）と `~/.cargo/registry/`（168M）は無事だった

[Issue #7](https://github.com/bosshawk/Harubridge/issues/7) はこの論点を調査し、
「Node と pnpm は mise、Rust は rustup」と結論して **ADR にはしない**としていた。
本 ADR はその結論を 2 点で覆す。

- **ADR にしないという判断を覆す。** 起票条件（[README](README.md#adr-にする条件)）のうち
  「覆すコストが高い」を満たさないとされたが、**pnpm の採用だけは満たす**。
  `pnpm-lock.yaml` は他のマネージャが読めず、[ADR-0005](0005-publish-as-oss.md) により
  リポジトリは公開される。加えて pnpm を選んだ理由そのもの（宣言していない依存を
  構造的に止める）は、npm に戻した瞬間に**規範ごと消える**。片方向の決定である
- **「Rust は mise に入れない」を覆す。** Issue #7 の根拠は
  「mise が `RUSTUP_TOOLCHAIN` を立て、`rust-toolchain.toml` を黙って上回る」だった。
  これは**両方に版を書いた場合にのみ成立する**問題であり、回避手段が存在することを後から確認した

### 調査した事実

いずれも 2026-08-09 に一次ソース、または手元の mise 2026.8.3 での実測により確認した。

**1. mise の rust は rustup のラッパーである**（[mise 公式 / Rust](https://mise.jdx.dev/lang/rust.html)）

> Rust/cargo can be installed which **uses rustup under the hood**. mise will install rustup
> if it is not already installed and install the requested toolchain, components, and targets.

実体は `~/.rustup` 配下に入り、`~/.local/share/mise/installs` には置かれない。
**mise は rustup の置き換えではなく、rustup を呼ぶ側である。**

**2. mise は `rust-toolchain.toml` を rust の idiomatic version file として読む**
（[mise 公式 / Configuration](https://mise.jdx.dev/configuration.html)）

対応表に `rust` → `rust-toolchain.toml` が載っている（拡張子なしの `rust-toolchain` は非対応）。
既定は無効で、`idiomatic_version_file_enable_tools` で有効化する。

**3. 読み取る項目は `channel` だけではない**（[`src/plugins/core/rust.rs`](https://github.com/jdx/mise/blob/main/src/plugins/core/rust.rs) の `parse_idiomatic_file`）

`channel` / `profile` / `components` / `targets` の 4 つを抽出する。
**`rust-toolchain.toml` に書けることは、mise 経由でもそのまま通る。**

**4. `idiomatic_version_file_enable_tools` は廃止されていない**
（[`settings.toml`](https://github.com/jdx/mise/blob/main/settings.toml)）

`deprecated` が付いているのは `legacy_version_file` / `legacy_version_file_disable_tools` /
`idiomatic_version_file` / `idiomatic_version_file_disable_tools` の 4 つで、
**そのすべてが `idiomatic_version_file_enable_tools` を移行先として名指ししている。**
この設定自体には `deprecated` も `hide` も付かない。

なお「deprecated」と紹介する記事が存在するのは、**既定値の反転**（`mise 2025.10.0` で
idiomatic version file が既定 有効 → 既定 無効に変わり、移行期間中は設定していない環境に
警告が出続けた）と、**設定名の 2 段階の改称**が混ざったためである。
本 ADR の構成は明示的に設定する側であり、警告は出ない。

**5. rustup の toolchain 選択の優先順位**（[rustup 公式 / Overrides](https://rust-lang.github.io/rustup/overrides.html)）

1. コマンドラインの `+toolchain` → 2. **`RUSTUP_TOOLCHAIN` 環境変数** →
3. ディレクトリ override → 4. **`rust-toolchain.toml`** → 5. 既定の toolchain

**mise は 2 番を立てる。**したがって両方に版を書けば、常に mise 側が勝つ。

**6. 実測（mise 2026.8.3 / macOS arm64）**

| 構成 | mise が解決した版 | 版の出所 |
| --- | --- | --- |
| `rust-toolchain.toml` + `enable_tools = ["rust"]` | `stable` | **`rust-toolchain.toml`** |
| 同上から `rust-toolchain.toml` を削除 | （解決しない） | — |
| 同上から設定を外す | （解決しない） | — |
| **`[tools] rust = "1.90.0"` も併記** | **`1.90.0`** | **`mise.toml`**（黙って勝つ） |

さらに、**mise の環境変数を外した状態**での `rustup show active-toolchain` は

```
stable-aarch64-apple-darwin (overridden by '/Users/don/Develop/Harubridge/rust-toolchain.toml')
```

を返した。**mise を経由しない経路（シェル外で起動したエディタなど）でも同じ版が選ばれる。**

**7. 採用規模**（[Issue #7](https://github.com/bosshawk/Harubridge/issues/7) が 2026-08-08 に
GitHub コード検索で実測。本 ADR では再測していない）

| ファイル | 件数 |
| --- | ---: |
| `rust-toolchain.toml` | **35,776** |
| `mise.toml`（`tools` を含む） | 21,120 |
| `mise.toml` + node | 9,264 |
| `mise.toml` + pnpm | 3,952 |
| `mise.toml` + rust | 2,920 |

## 決定

**開発ツールチェインの導入口を mise に統一する。ただし Rust の版は `mise.toml` に書かず、
`rust-toolchain.toml` を唯一の正とし、mise にそれを読ませる。**

リポジトリのルートに増えるのは `mise.toml` と `rust-toolchain.toml` の 2 つだけである。

```toml
# mise.toml
[settings]
idiomatic_version_file_enable_tools = ["rust"]

[tools]
node = "26.7.0"
pnpm = "11.20.0"
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

- **`mise.toml` の `[tools]` に `rust` を書かない。** 書いた瞬間に上記の事実 5・6 が発動し、
  `rust-toolchain.toml` が黙って無視される
- **Node と pnpm はパッチまで完全固定する。** `Cargo.lock` / `pnpm-lock.yaml` をコミットし、
  `tauri-specta` を `=` で固定する（[ADR-0018](0018-dependencies.md)）既存方針と揃える
- **Rust は `stable` に留める。** パッチ版固定は全員に巨大なダウンロードを強制する
- **`.nvmrc` を作らない。** 読み手が mise しかいないなら `mise.toml` に直接書けば済む
- **CI は [`jdx/mise-action`](https://github.com/jdx/mise-action) を使い、
  ローカルと同じファイルを同じ道具に読ませる**（CI 自体の整備は未着手）

## 検討した選択肢

### 論点 1: Node のパッケージマネージャ

#### 案 A1: pnpm（採用）

- 概要: `pnpm-lock.yaml` をコミットする。`.npmrc` は必要になるまで置かない
  （Tauri 本体も置いていない）
- 利点:
  - **Tauri を使う場面のデファクトである。** Issue #7 の実測では
    `tauri-apps/tauri` 本体が pnpm 11.13.0、GitButler が 10.20.0、
    艦これ / Tauri v2 の `tsukasa-u/FUSOU` が 10.14.0、Clash Verge Rev が 11.3.0
  - **宣言していない依存をうっかり使う事故を構造的に止める。**
    [ADR-0003](0003-agent-driven-development.md) の体制ではこの事故が起きやすく、
    しかも**動いてしまうため誰も気づかない**
- 欠点: `pnpm-lock.yaml` は他のマネージャが読めない。乗り換えは片方向になる

#### 案 A2: npm

- 概要: Node 同梱で追加の導入が要らない
- 却下理由: Tauri 公式の Prerequisites が npm を例示するのみで、
  **Tauri チーム自身のリポジトリを含む実プロジェクト 4/5 が pnpm だった**ため
  （文面ではなく実態で測った結果）。

#### 案 A3: bun

- 却下理由: ランタイムごと差し替わる。Tauri 公式が無言及で、Issue #7 が調べた実例でも
  Spacedrive のみの採用だったため。

#### 案 A4: yarn

- 却下理由: pnpm・npm に対する優位が見つからなかったため。

### 論点 2: 版を固定する道具

#### 案 B1: mise（採用）

- 概要: Node / pnpm を mise で管理し、Rust も mise 経由で導入する
- 利点:
  - **管理者が不在だった Node と pnpm に管理者ができる。** `.nvmrc` も
    `packageManager` フィールドも、それ自体は何も実行しない
  - **`mise install` 一発で環境が揃う。** rustup の導入も mise が引き受ける
    （実測で確認。既存の `~/.rustup` は再利用され、1.2G の再取得は発生しなかった）
  - CI で同じファイルを同じ道具に読ませられる（`jdx/mise-action`）
- 欠点: mise の導入自体が新しい前提になる。README に手順が要る

#### 案 B2: asdf

- 却下理由: mise が上位互換であり、mise を選ばない理由が見つからなかったため。

#### 案 B3: proto

- 却下理由: star 1,374 と規模不足。[ADR-0003](0003-agent-driven-development.md) の体制では
  参照できる前例の量がそのまま実装可能性に効くため。

#### 案 B4: Volta

- 却下理由: Issue #7 の調査時点（2026-08-08）で最終 push が 2025-11 と停滞していたため。

### 論点 3: Rust の版をどこに書くか

**3 案すべてで `mise` と `rust-toolchain.toml` が「両方存在しうる」。差は版の正がどこにあるかである。**

#### 案 C1: `rust-toolchain.toml` を正とし、mise に読ませる（採用）

- 概要: `mise.toml` に `idiomatic_version_file_enable_tools = ["rust"]` を置き、
  `[tools]` には rust を書かない。版・components・targets は `rust-toolchain.toml` に書く
- 利点:
  - **版の記述が 1 箇所に収まる。** 同期の手作業が発生しない
  - **mise を経由しない経路でも同じ版が選ばれる**（事実 6 の `overridden by` を実測）。
    シェル外で起動したエディタの rust-analyzer がズレない
  - **rustup 側から見れば普通の `rust-toolchain.toml` プロジェクトである。**
    採用規模で最も厚い形（35,776 件）に乗れる。mise を持たない外部貢献者にも版が効き、
    [ADR-0005](0005-publish-as-oss.md) の未解決事項（外部貢献の受け入れ）に余地を残す
  - `RUSTUP_TOOLCHAIN` は立つが、**中身が `rust-toolchain.toml` と同一なので無害**
- 欠点:
  - idiomatic version file は mise が**既定で無効にしている**機構であり、
    mise の版を上げるときに挙動を確認する対象が 1 つ増える
  - `mise.toml` の `[tools]` に rust が現れないため、
    **mise が rust も管理していることが `mise.toml` だけ見ても分かりにくい**。コメントで補う

#### 案 C2: `mise.toml` に版を書き、`rust-toolchain.toml` も別に置く

- 概要: 両方に版を書き、手作業で同期する
- 却下理由: **ズレたときに `mise.toml` が黙って勝つ**（事実 5・6 で実測）。
  二重管理であり、しかも不一致が沈黙のまま進行するため。

#### 案 C3: `mise.toml` に版を書き、`rust-toolchain.toml` を作らない

- 概要: 版の正を `mise.toml` に一本化する
- 却下理由: mise の環境変数が届かない経路（シェル外で起動したエディタ、
  mise を持たない利用者）で版が固定されなくなるため。
  採用規模の薄い側（2,920 件）に乗る不利も残る。

#### 案 C4: mise は rust に触れず、rustup と `rust-toolchain.toml` だけで管理する

- 概要: [Issue #7](https://github.com/bosshawk/Harubridge/issues/7) の当初の結論
- 却下理由: **`mise install` 一発で環境が揃わず、rustup の導入が別手順として残る**ため。
  オーナー環境で rustup の取り次ぎ口が壊れていた事象が、まさにその別手順の脆さだった。

## 決め手

**版の正を 1 箇所に保ったまま導入口を 1 つにするために、
mise の既定で無効な機構（idiomatic version file）に乗ることを受け入れた。**

## 影響

- 実装への影響:
  - リポジトリのルートに `mise.toml` と `rust-toolchain.toml` が増える。
    [ADR-0027](archive/0027-repository-layout.md) のツリー図はこの 2 ファイルを含まないが、
    同 ADR は `Proposed` であり本 ADR が差分を記録する（同 ADR の本文は変更しない）
  - リポジトリの `mise.toml` は初回に **`mise trust` が必要**（`mise` の既定の安全機構）
  - エージェントが叩くコマンドは変わらない —— `pnpm install` / `pnpm tauri dev` /
    `pnpm lint` / `cargo test` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check`
  - **Rust の補助ツール（`cargo-deny` 等）は入れない。** 走らせる CI がまだ無い
- ドキュメントへの影響:
  - `README.md` に mise の導入手順を書く（[C-07](../spec/constraints.md) により
    リポジトリは公開されるため、第三者が環境を再現できる必要がある）
  - `docs/spec/` と `docs/guidelines/` への影響は無い。
    ツールチェインは[ADR-0008](0008-code-as-source-of-truth.md) の言う「基本設計」ではなく、
    設定ファイルそのものが正である
- 取り消す場合のコスト:
  - Rust の管理方法（案 C1 ↔ C3 ↔ C4）の切り替え: **低**。設定 1 行とファイル 1 枚
  - mise をやめる: **低**。`mise.toml` を消し、各ツールを個別に導入する手順に戻す
  - **pnpm をやめる: 中。** `pnpm-lock.yaml` が失われ、
    宣言していない依存を止める構造も同時に失われる。実装が進むほど高くなる

## 未解決事項

- `TODO(未確定)`: CI の構成そのもの。`jdx/mise-action` を使う方針だけが決まっており、
  ジョブの中身は未着手（[ADR-0027](archive/0027-repository-layout.md) / [ADR-0022](0022-observed-data-privacy.md)
  も CI 未整備を未解決事項として記録している）
- `TODO(未確定)`: `.gitattributes`（改行コード）。Windows 対応（[Issue #6](https://github.com/bosshawk/Harubridge/issues/6)）
  に着手する時点で判断する
- `TODO(未確定)`: npm scripts に環境変数の指定や `rm -rf` を書くと Windows で壊れる。
  クロスプラットフォームな記述に限る規約が要る（[docs/guidelines/](../guidelines) の範囲）
- `TODO(未確定)`: `protoc` のような**自前で版管理しない道具**が増えたら `mise.toml` に足す
- `TODO(要検証)`: mise の版を上げたときに idiomatic version file の挙動が変わらないこと。
  上記の実測は **mise 2026.8.3** で行った。**版を上げる際は `mise ls --current` の
  「出所」列が `rust-toolchain.toml` のままであることを確認する**
