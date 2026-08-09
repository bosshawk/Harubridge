# ADR-0034: CI を GitHub Actions に置き、OS 非依存の検査は ubuntu、OS 依存のものだけ macOS / Windows で回す

- ステータス: **Proposed**
- 日付: 2026-08-09
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0022](0022-observed-data-privacy.md)（**判定の正を CI に置くと決めている。本 ADR はその実装**）,
  [ADR-0031](0031-toolchain-management.md)（mise / `jdx/mise-action`）,
  [ADR-0033](0033-task-runner.md)（`task check` は CI と同じ内容にする）,
  [ADR-0032](0032-repository-structure.md)（§6 生成物の差分ゼロを CI で強制する）,
  [ADR-0014](0014-trunk-based-on-main.md)（`main` 直コミット。PR が無い）,
  [ADR-0005](0005-publish-as-oss.md)（public リポジトリ）,
  [ADR-0019](0019-linter.md), [ADR-0016](0016-tech-stack.md),
  [docs/spec/constraints.md](../spec/constraints.md)（C-03 / C-04 / C-07）,
  [docs/spec/overview.md](../spec/overview.md)（対応 OS）

## 背景と課題

**CI が 1 つも無い。** `.github/` が存在しない。

これは単に「まだ作っていない」では済まない。既に承認済みの決定が複数、CI の存在を前提にしている。

| 決定 | CI に何を求めているか |
| --- | --- |
| [ADR-0022](0022-observed-data-privacy.md) 決定 3 | フィクスチャが生成器の出力とバイト一致しなければ fail |
| [ADR-0022](0022-observed-data-privacy.md) 具体策 3 | `svdata=` で始まる、または `api_result` と `api_data` を同時に持つ JSON が生成器の出力以外に存在したら fail |
| [ADR-0022](0022-observed-data-privacy.md) 具体策 4 | gitleaks を置き、`api_token` を検出するカスタムルールを足す |
| [ADR-0032](0032-repository-structure.md) §4 / §6 | `kcsapi-hook.js` と `bindings.ts` を再生成して差分ゼロを強制 |
| [ADR-0033](0033-task-runner.md) | `task check` は CI と同じ内容にする |

とくに [ADR-0022](0022-observed-data-privacy.md) は、
**「`.local/` を使い始める前に CI の判定を入れる。順序は CI が先である」**と明記している。
つまり **CI が無い限り実装に着手できない。** 現在そこで止まっている。

[ADR-0022](0022-observed-data-privacy.md) 自身も未解決事項に
「`TODO(未確定)`: gitleaks のカスタムルールの具体的な内容と、CI の構成そのもの（CI 自体が未整備）」
を残しており、本 ADR はその宿題を引き取る。

### 前提として既に決まっていること（本 ADR は覆さない）

- **CI は [`jdx/mise-action`](https://github.com/jdx/mise-action) を使い、
  ローカルと同じファイルを同じ道具に読ませる**（[ADR-0031](0031-toolchain-management.md) の決定に含まれる）。
  したがって「`setup-node` / `pnpm/action-setup` / `dtolnay/rust-toolchain` を並べるか」は
  本 ADR で改めて比較しない
- **判定の正はフックではなく CI に置く**（[ADR-0022](0022-observed-data-privacy.md) 具体策 5）。
  `.gitignore` は `git add -f` で、フックは無効化で越えられるため
- **対応 OS は macOS / Windows**（[overview.md](../spec/overview.md)）。Linux は対象外

### 調査した事実

いずれも 2026-08-09 に確認した。

**GitHub Actions の実行環境。**

- **public リポジトリは標準ランナーを無償・無制限で使える**
  （[GitHub Docs: GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)）。
  本プロジェクトは public（[ADR-0005](0005-publish-as-oss.md)）であり、**OS 台数は費用の制約にならない**
- ラベルの現在の対応: `macos-latest` → arm64 の macOS 15、`windows-latest` → `windows-2025`（同上）

**`jdx/mise-action` の現況。**

- 現行のメジャー版は **v4**。`- uses: jdx/mise-action@v4` の 1 行で mise を導入し、
  既定で `mise install` まで行う。ツールのキャッシュを既定で持つ
  （[README](https://github.com/jdx/mise-action)）
- `mise.toml` に `task = "3.52.0"` を書いてあるため、**`mise install` だけで `task` も入る。**
  CI 側に go-task の導入手順を書く必要がない

**gitleaks のライセンス。**

- **CLI 本体（[gitleaks/gitleaks](https://github.com/gitleaks/gitleaks)）は MIT。**
  `dir` サブコマンドで git 履歴に依らずディレクトリ・ファイルを走査できる
- **公式の Action（[gitleaks/gitleaks-action](https://github.com/gitleaks/gitleaks-action)）は
  v2.0.0 で MIT から独自ライセンスに変更された。**
  README は「組織アカウントのリポジトリを走査するならライセンスキー（無償）が要る。
  個人アカウントのリポジトリならキーは不要」と述べる
- `mise registry` に `gitleaks`（`aqua:gitleaks/gitleaks`）が存在する。
  実行環境で `mise registry | grep gitleaks` により確認した

**Windows ランナーの改行変換。**

- Windows では git の `core.autocrlf` が既定で `true` であり、
  `actions/checkout` はチェックアウト時に LF を CRLF に変換する
  （[actions/checkout#135](https://github.com/actions/checkout/issues/135)、
  [GitHub Docs: Configuring Git to handle line endings](https://docs.github.com/en/get-started/git-basics/configuring-git-to-handle-line-endings)）
- **本リポジトリに `.gitattributes` が無い**（`git ls-files` で確認）。
- Prettier の `endOfLine` の既定は **`lf`**（v2.0.0 以降）。
  公式ドキュメントは「リポジトリを LF だけに保つ」手順として
  `endOfLine: lf` + フック + **CI で `--check`** を並べている
  （[Prettier: Options](https://prettier.io/docs/options)）。
  つまり **CRLF でチェックアウトされた Windows では `pnpm format:check` が落ちる。**
  `.prettierrc` は `{}` で、既定のまま
- 対して `.editorconfig` は `end_of_line = lf` を宣言している。**宣言と実際の運用が食い違う。**

**Tauri の CI に関する公式の案内。**

- 推奨ランナーは macOS が `macos-latest`、Windows が `windows-latest`、Linux が `ubuntu-22.04`。
  **追加のシステムパッケージが要るのは Linux だけ**（`libwebkit2gtk-4.1-dev` ほか）。
  Rust のキャッシュに `Swatinem/rust-cache` を使うことを勧めている
  （[Tauri v2: GitHub Actions](https://v2.tauri.app/distribute/pipelines/github/)）
- `tauri build` に **`--no-bundle`**（「`bundle > active` が `true` でもバンドル工程を飛ばす」）がある
  （[Tauri v2: CLI reference](https://v2.tauri.app/reference/cli/)）

**OS によって結果が変わる検査と、変わらない検査（リポジトリの実物を読んで確認した）。**

- **`crates/harubridge-core/` は `tauri` に依存しない**（`Cargo.toml` で確認。
  [ADR-0032](0032-repository-structure.md) §1 が強制している境界）。
  したがって **Linux で `libwebkit2gtk-4.1-dev` 等を入れなくても clippy とテストが通る**
- **`src-tauri/` は `tauri` に依存する。** ここだけが Linux で追加パッケージを要求する
- **[ADR-0029](0029-injection-ipc-transport.md)（Proposed）が採ろうとしている実装は、
  macOS と Windows で完全に別のコードである** —— macOS は `WKUserContentController` の
  `addScriptMessageHandler:name:`、Windows は COM の `ICoreWebView2Frame2::add_WebMessageReceived`。
  **`#[cfg(target_os = ...)]` で分岐したコードは、その OS 向けにコンパイルしない限り
  型検査すらされない。** Linux でビルドしても、どちらの分岐も消える
- **Linux 版の実装は存在しない**（対応 OS でない）。
  `cargo clippy --workspace` を Linux で通すには、**誰も使わない Linux 分岐をコード側に
  維持し続ける**ことになる
- **Linux のファイルシステムは大文字小文字を区別する。**
  macOS と Windows の既定は区別しない。したがって import パスの大小のずれは
  **Linux でだけ検出できる**（`tsc` / `eslint` の検出力が上がる）
- **実行時の差異（WebView の挙動・通知・ウィンドウ）は、どの OS で CI を回しても検証できない。**
  E2E も GUI テストも存在しないため。Windows の実機確認は Issue #6 に残っており、
  **CI はそれを代替しない**

**現在の `Taskfile.yml` の穴（実ファイルを読んで確認）。**

- `check:fe` の `desc` は「型・リント・整形・**注入スクリプトの差分**」と書いているが、
  `cmds` は `tsc --noEmit` ×2 / `pnpm lint` / `pnpm format:check` のみで、
  **差分の検査を一切していない**（[ADR-0032](0032-repository-structure.md) §6 が要求している検査が未実装）
- `src-tauri/injected/kcsapi-hook.js` は `git ls-files` に現れる。**生成物が追跡されている**

## 決定

**CI を GitHub Actions に置く。ワークフローに書くのは
「ツールチェインを入れる」と「`task` のタスクを呼ぶ」の 2 つだけとし、
検査の中身は Taskfile と cargo / package.json 側に置いて CI 固有の記述を作らない。
実行環境は「その検査は OS によって結果が変わりうるか」の一点で振り分け、
変わらないものは `ubuntu-latest` に集約する。**

- ワークフローは `.github/workflows/ci.yml` の 1 本。
  トリガーは `push`（`main`）/ `pull_request` / `workflow_dispatch`
- ジョブは 2 つ

| ジョブ | 環境 | 内容 |
| --- | --- | --- |
| **静的検査** | `ubuntu-latest` **のみ** | 型検査・リント・整形・生成物の差分・観測データの混入判定・gitleaks・`cargo fmt --check`・**`tauri` に依存しないクレート**の clippy とテスト |
| **プラットフォーム検査** | `macos-latest` / `windows-latest` の matrix | `src-tauri` を含む `cargo clippy --workspace`・`cargo test`・`tauri build --no-bundle` |

- **振り分けの基準は 1 本だけ。「OS によって結果が変わりうるか」。**
  変わらないもの（構文・型・書式・秘密情報・リポジトリの形）は 1 回だけ回す。
  変わりうるもの（`#[cfg(target_os)]` 分岐のコンパイル、ファイルパス・改行・時刻を触るコードの実行）は
  対応 OS の 2 つで回す
- **この線は既存のクレート境界と一致する。** `crates/harubridge-core/` は `tauri` を知らず
  （[ADR-0032](0032-repository-structure.md) §1 が強制している）、Linux でそのまま検査できる。
  `src-tauri/` だけが OS に触れる
- **プラットフォーム検査の主目的は、OS 別コードのコンパイルである。**
  [ADR-0029](0029-injection-ipc-transport.md) が採ろうとしている macOS / Windows の実装は互いに別物であり、
  **その OS でコンパイルしない限り型検査すら行われない。**
  とくに Windows は一度も動かしていない（Issue #6）ため、
  **コンパイルが通ることの唯一の自動的な証拠**になる
- バンドル・署名・公証は本 ADR の対象外（`--no-bundle`）
- **CI から呼ぶコマンドはすべて `task <name>` の形にする。**
  必要なタスクが無ければ Taskfile に足す。**ワークフローの YAML に生のコマンドを書かない**
- ツールチェインは `jdx/mise-action@v4`。Rust のビルドキャッシュに `Swatinem/rust-cache` を使う
- **観測データの混入判定（[ADR-0022](0022-observed-data-privacy.md) 具体策 3）は、
  ワークスペース内の Rust の実行可能クレートとして実装する。**
  走査対象は `git ls-files` が返す追跡対象ファイル。判定自体に単体テストを付ける
- **gitleaks は公式 Action を使わず、mise で CLI を入れて `task` から呼ぶ**
- **`.gitattributes` を追加し、`* text=auto eol=lf` を宣言する**

## 検討した選択肢

### 論点 1: CI を置くか

#### 案 A1: GitHub Actions に置く（採用）

- 概要: リポジトリと同じ場所で回す。public のため標準ランナーは無償・無制限
- 利点:
  - [ADR-0022](0022-observed-data-privacy.md) が「判定の正」と名指しした場所を実際に用意できる。
    ここが無い限り `.local/` を使えず、実装に着手できない
  - 費用が制約にならないため、**OS の台数やジョブの分割を費用で妥協せずに決められる**
- 欠点: [ADR-0014](0014-trunk-based-on-main.md) の下では PR が無いため、
  **検出は常に push のあと**になる（下記「決め手」と「影響」）

#### 案 A2: CI を置かず、ローカルの `task check` だけに頼る

- 概要: 規約で「push 前に `task check` を回す」と定めるだけにする
- 利点: 何も増えない。CI の待ち時間が無い
- 却下理由: [ADR-0022](0022-observed-data-privacy.md) が
  「`.gitignore` もフックも越えられるため判定の正は CI に置く」と決めており、
  それを覆さない限り `.local/` を使い始められないため。

> CI サービスそのものの比較（GitLab CI ほか）は行っていない。
> リポジトリを GitHub でホストしており（[ADR-0005](0005-publish-as-oss.md)）、
> public リポジトリでは無償・無制限であるため、他所へ持ち出す理由が無い。

### 論点 2: どの OS で回すか

#### 案 B1: OS 非依存の検査は ubuntu、OS 依存のものだけ macOS / Windows（採用）

- 概要: 「OS によって結果が変わりうるか」で振り分ける。
  `tauri` に依存しない部分はすべて `ubuntu-latest` に集約し、
  `src-tauri` を含む部分だけ対応 OS 2 つで回す
- 利点:
  - **同じ検査を 3 回回さない。** 型・リント・整形・秘密情報の検査に OS の意味は無い
  - **Linux は大文字小文字を区別するため、静的検査の検出力がむしろ上がる。**
    import パスの大小のずれは macOS / Windows では素通りする
  - ubuntu は 3 つの中で最も速く、**壊れたことに気づくまでの時間が短くなる**。
    [ADR-0014](0014-trunk-based-on-main.md) の下では検出が常に push のあとになるため、ここは効く
  - **振り分けの線が既存のクレート境界と一致する**ため、どちらに置くかで迷わない。
    `tauri` に依存するものが増えたら、それはプラットフォーム検査側である
  - `src-tauri` を Linux でビルドしないため、**Tauri の Linux 用システムパッケージを入れずに済む**
- 欠点:
  - ジョブが 2 種類になり、ワークフローが「全部同じ」より複雑になる
  - **対応 OS でない環境でだけ落ちる**ことが起きうる（例: パスの大小）。
    ただしそれは検出力が上がった結果であり、直す価値のある誤りである

#### 案 B2: すべての検査を `macos-latest` + `windows-latest` の 2 つで回す

- 概要: 対応 OS と CI の実行環境を完全に一致させ、ジョブを 1 種類に保つ
- 利点: ワークフローが単純。「CI が緑 = 対応 OS で緑」が一言で言える
- 却下理由: **静的検査には OS の意味が無いのに、実行時間だけが倍になる**ため。
  `tsc` や `eslint` が macOS で緑になったことは、対応 OS について何も保証しない。
  加えて大文字小文字を区別しない 2 OS だけになり、**Linux で回せば見つかる誤りを取り逃がす。**

#### 案 B3: `macos-latest` のみ

- 概要: 実測済みの OS 1 つに絞る
- 却下理由: 対応 OS に挙げている Windows が、**コンパイルできるかすら誰も知らない**状態が続くため。
  [ADR-0029](0029-injection-ipc-transport.md) の Windows 実装は macOS 版と別のコードであり、
  macOS でビルドしても `#[cfg]` により消える。

#### 案 B4: `ubuntu-latest` のみ

- 概要: 最も速く安い 1 つに集約する
- 却下理由: **OS 別コードが一切コンパイルされない**ため。
  加えて Linux 版の実装は存在しないので、`cargo clippy --workspace` を Linux で通すには
  **誰も使わない Linux 分岐をコード側に維持し続ける**ことになる。

### 論点 3: 観測データの混入判定をどう実装するか

[ADR-0022](0022-observed-data-privacy.md) 具体策 3 の判定
（`svdata=` で始まる、または `api_result` と `api_data` を同時に持つ JSON が
生成器の出力以外に存在したら fail）を、何で書くか。

#### 案 C1: ワークスペース内の Rust の実行可能クレート（採用）

- 概要: `crates/` に小さな実行可能クレートを 1 つ足し、`git ls-files` の出力を走査する。
  Taskfile から呼び、CI もそれを呼ぶ
- 利点:
  - **判定そのものに単体テストを書ける。** 判定が壊れていれば、
    それは「壊れていることに気づけない防御」であり、無いより悪い
  - `cargo clippy --workspace --all-targets` と `cargo test` の対象に自動的に入る
  - **OS 依存が無い。** 2 OS の matrix でそのまま動く
  - 除外条件である「生成器の出力」は [ADR-0022](0022-observed-data-privacy.md) により
    Rust の生成器が決める。**判定と生成器を同じ言語・同じワークスペースに置けば、
    出力パスの定義を 1 箇所で共有できる**
- 欠点: ワークスペースのメンバーが 1 つ増える。
  アプリの機能ではないコードが `crates/` に混ざる

#### 案 C2: `harubridge-core` の結合テストとして書く

- 概要: `crates/harubridge-core/tests/` に置き、`cargo test` で回す
- 利点: 新しいクレートが増えない。テストの体裁が自然
- 却下理由: コアの責務は通信データの解釈と状態保持であり
  （[ADR-0032](0032-repository-structure.md) §1）、**リポジトリの衛生検査は無関係な責務**のため。
  テストからリポジトリ根を相対パスで辿る形にもなり、クレートの独立性を壊す。

#### 案 C3: シェルスクリプト / Node スクリプト

- 概要: `scripts/` に置いて Taskfile から呼ぶ
- 却下理由: 判定に単体テストを付けにくく、Windows での挙動を別に確かめる必要が出るため。
  「JSON が 2 つのキーを同時に持つ」の判定を正規表現で書けば脆く、
  パーサを使えば結局スクリプトが育つ。

#### 案 C4: gitleaks のカスタムルールで表現する

- 概要: 判定を gitleaks の TOML ルールとして書き、道具を 1 つに減らす
- 却下理由: gitleaks は**秘密情報の検出器**であり、目的（形による混入の検出）と対象
  （追跡対象ファイル全体）が異なるため。TOML の正規表現には単体テストを書けず、
  除外条件である「生成器の出力」を gitleaks の allowlist 側にも二重に定義することになる。

### 論点 4: gitleaks をどう入れるか

#### 案 D1: mise で CLI を入れ、`task` から呼ぶ（採用）

- 概要: `mise.toml` の `[tools]` に `gitleaks` を足す。CI もローカルも同じコマンドを叩く
- 利点:
  - **CLI 本体は MIT** で、[ADR-0011](0011-license-mit.md) と同じライセンス系列に収まる
  - **ローカルで同じ検査を回せる。** push する前に自分で確かめられる
  - ツールの導入経路が [ADR-0031](0031-toolchain-management.md) の mise 一本に揃う
- 欠点: 版の更新を自分で追う必要がある（`mise.toml` に版を書くため、既存のツールと同じ扱い）

#### 案 D2: 公式の `gitleaks/gitleaks-action` を使う

- 概要: Marketplace の Action を 1 行足す
- 利点: 導入が最短。SARIF の GitHub への連携が用意されている
- 却下理由: **v2.0.0 以降ライセンスが MIT から独自ライセンスに変わっており**、
  個人アカウントでは無償だが、リポジトリの所有者が変われば条件が変わる。
  加えて **CI でしか動かせず、ローカルで同じ検査を回せない**ため。

### 論点 5: Windows の改行変換にどう対処するか

#### 案 E1: `.gitattributes` に `* text=auto eol=lf` を置く（採用）

- 概要: リポジトリ側で改行を宣言し、チェックアウト時の変換を止める
- 利点:
  - **CI だけでなく、Windows の開発機にも同じように効く。**
    Windows で開発を始めた瞬間に `pnpm format:check` が落ちる、という事故が起きない
  - `.editorconfig` の `end_of_line = lf` という**既存の宣言と実際の運用が一致する**
  - GitHub の公式ドキュメントが勧めている形であり、追加の道具が要らない
- 欠点: 既にチェックアウト済みの作業ツリーでは、
  一度 `git add --renormalize .` に相当する操作が要る場合がある

#### 案 E2: ワークフローで `git config --global core.autocrlf false` を実行する

- 概要: `actions/checkout` の前に 1 ステップ足す
- 却下理由: **CI だけが直り、Windows の開発機は落ちたまま**になるため。
  加えて「CI 固有の記述を作らない」という本 ADR の方針に反する。

## 決め手

**「CI で落ちたものは手元でも落ちる」という再現性を得るために、
CI 専用に最適化する自由を捨てた。**

CI にしか無いコマンドは、必ずローカルで再現できない失敗を生む。
コードを書くのがエージェントであり、人間が全行を見張らない前提
（[ADR-0003](0003-agent-driven-development.md)）では、
**「CI で落ちたが手元では再現しない」状態が最も高くつく。**

OS の振り分けも同じ基準に従っている。速さや費用ではなく
**「その検査は OS によって結果が変わりうるか」**だけで決めており、
変わらないものを 3 回回しても再現性は 1 ミリも増えない。
なお費用は判断材料にならない（public リポジトリは標準ランナーが無償・無制限）。

## 影響

### 実装への影響

- 新規に作るもの:
  - `.github/workflows/ci.yml`（静的検査ジョブ 1 つ + プラットフォーム検査ジョブの 2 OS matrix）
  - `.gitattributes`（`* text=auto eol=lf`）
  - 観測データの混入判定を行う実行可能クレート（[ADR-0032](0032-repository-structure.md) の
    ツリーに 1 メンバー追加。**同 ADR の「強制される境界は 3 本」には影響しない**）
- `mise.toml` の `[tools]` に `gitleaks` を追加する
- **`Taskfile.yml` に足りない検査を足し、`tauri` 依存の有無で呼び分けられるように割る**:
  - 生成物の差分検査（`kcsapi-hook.js` / `bindings.ts` を再生成して差分ゼロ）。
    **`check:fe` の `desc` が「注入スクリプトの差分」と書いているのに実装されていない**穴を埋める
  - 観測データの混入判定
  - gitleaks
  - `tauri build --no-bundle`
  - 現在の `check:rust` は `cargo clippy --workspace` であり、これは `src-tauri` を含む
    （＝プラットフォーム検査側）。**ubuntu 用に「`tauri` に依存しないクレートだけ」の
    入口を別に用意する。** 開発者が手元で叩く `task check` は、
    従来どおり自分の OS で全部を回す
- **静的検査を ubuntu だけで回すため、`pnpm format:check` は CI で一度も Windows を踏まない。**
  Windows の改行変換に対する防御は `.gitattributes` の一本に集約される
  （それが論点 5 で「CI 側の `core.autocrlf` 設定」を却下した理由でもある）
- `cargo` を回す時点で `crates/harubridge-core/build.rs` が走り、
  `data/kancolle/*.json` の JSON としての妥当性は自動的に検査される（既存の実装）
- **`bindings.ts` はまだ存在しない**（`tauri-specta` の配線が未実装）。
  差分検査の対象に加えるのは、生成が動くようになってからでよい

### `main` 直コミット（[ADR-0014](0014-trunk-based-on-main.md)）との関係

[ADR-0014](0014-trunk-based-on-main.md) は再検討の契機の 1 つに
「CI を導入し、マージ前の自動検証が必要になった」を挙げている。
**本 ADR はこの契機に該当しない。** PR を作らないため「マージ前」という時点が存在せず、
CI は push のあとに回る**事後検出**になる。

これは [ADR-0022](0022-observed-data-privacy.md) が既に受け入れている構図である
（フックは無効化できるため、判定の**正**を CI に置くという決定）。
ただし **public リポジトリでは、事後検出では取り返しがつかないものがある**（C-07: 履歴から消せない）。
そこで防止は次の 2 段構えとする。

1. **防止（ローカル）** —— push 前に検査系のタスクを回す。**規約であり、保証ではない**
2. **保証（CI）** —— 越えられない場所で必ず落とす。**ただし落ちた時点で既に push されている**

**1 を [CLAUDE.md](../../CLAUDE.md) §5 の完了条件に加える必要がある**（人間の承認が要る）。

### ドキュメントへの影響

- [CLAUDE.md](../../CLAUDE.md) §5「作業の完了条件」に、push 前の検査を 1 行足す（**承認が要る**）
- [CONTRIBUTING.md](../../CONTRIBUTING.md) に CI の説明と、落ちたときの直し方を足す
- `docs/spec/` の変更は無い。**CI はユーザーに見える振る舞いではない**
- `docs/guidelines/` への追記も本 ADR では行わない
  （テストとフィクスチャの規約は [ADR-0022](0022-observed-data-privacy.md) が別途 1 本を要求している）

### 取り消す場合のコスト

**低い。** ワークフローの削除・OS の増減・ジョブの分割はいつでも変えられる。
検査の中身が Taskfile 側にあるため、CI サービスを移す場合も
「ツールチェインを入れて `task` を呼ぶ」を書き直すだけで済む。

**`.gitattributes` の追加だけは影響が広い**（作業ツリー全体の再正規化が起きうる）。
ただし現在すべてのファイルが LF でコミットされているため、実際の差分は生じない見込みである。

## 未解決事項

- **フィクスチャの生成器（[ADR-0022](0022-observed-data-privacy.md) 決定 3）が未実装である。**
  したがって混入判定の「生成器の出力は除外する」条件は、現時点では**除外対象ゼロ**として実装する。
  生成器を作る時点で除外の定義を埋める。
  同 ADR の未解決事項「生成器の実装形態（`cargo test` 内か独立した bin か）」も、そこで決める
- **Git フック（[ADR-0022](0022-observed-data-privacy.md) 具体策 4 の pre-commit）を本 ADR では入れない。**
  lefthook / prek などの導入は依存の追加であり、
  [ADR-0018](0018-dependencies.md) の系列で別途決める。
  **判定の正が CI にあることは変わらないため、実装着手は妨げない**
- `TODO(要検証)`: **`tauri build` が Windows で通るかは未確認。** CI を入れて初めて分かる。
  通らなかった場合の対処（Issue 化するか、ビルドジョブを一時的に外すか）はその時点で判断する
- `TODO(未確定)`: gitleaks のカスタムルールの具体的な内容。
  `api_token` を検出するルールを足すことは [ADR-0022](0022-observed-data-privacy.md) が決めているが、
  **`api_token` の実際の形式（長さ・文字種）を観測できていない**ため、
  現時点では既定ルールのみで導入し、実測後にカスタムルールを足す
- 配布（署名・公証・自動更新）とリリースの自動化は本 ADR の対象外。
  外部仕様と配布方法が決まってから別に起票する
- **プラットフォーム検査は、現時点ではほとんど空回りする。**
  `src-tauri/src/lib.rs` はまだ雛形であり、OS 別コードは 1 行も無い。
  効き始めるのは [ADR-0029](0029-injection-ipc-transport.md) が承認され、
  `#[cfg(target_os = ...)]` の実装が入ってからである。
  **先に置く理由は、そのコードが入った初日から型検査されている状態にしておくためである**
- Linux を対応 OS に加えることは検討していない。加えるなら CI ではなく
  [overview.md](../spec/overview.md) の変更（＝仕様の変更）が先である
- CI の実行時間が問題になった場合の追加の分割は、実際に遅くなってから判断する。
  **先に最適化しない**
