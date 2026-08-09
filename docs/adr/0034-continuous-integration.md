# ADR-0034: 検査のための CI を GitHub Actions の ubuntu 1 ジョブに置く

- ステータス: Accepted
- 日付: 2026-08-09
- 決定者: プロジェクトオーナー
- 関連: [ADR-0022](0022-observed-data-privacy.md)（**判定の正を CI に置くと決めている。本 ADR はその実装**）,
  [ADR-0031](0031-toolchain-management.md)（mise / `jdx/mise-action`）,
  [ADR-0033](0033-task-runner.md)（`task check` は CI と同じ内容にする）,
  [ADR-0032](0032-repository-structure.md)（§1 コアは `tauri` を知らない / §6 生成物の差分ゼロを CI で強制する）,
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
  本プロジェクトは public（[ADR-0005](0005-publish-as-oss.md)）であり、
  **OS 台数は費用の制約にならない。したがって「何台まわすか」は費用ではなく、
  得られるものだけで決める**
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

**ubuntu で回せるもの・回せないもの。**

- **`crates/harubridge-core/` は `tauri` に依存しない**（`Cargo.toml` で確認。
  [ADR-0032](0032-repository-structure.md) §1 がコンパイラに強制させている境界）。
  **Linux で追加パッケージ無しに clippy とテストが通る**
- **`src-tauri` は `tauri` に依存する。** Tauri は Linux でだけ
  `libwebkit2gtk-4.1-dev` / `libappindicator3-dev` / `librsvg2-dev` / `patchelf` / `xdg-utils` を要求する
  （[Tauri v2: GitHub Actions](https://v2.tauri.app/distribute/pipelines/github/)。
  macOS / Windows には追加のシステム依存が無い）。
  つまり **ubuntu で `src-tauri` をビルドすることは、対応 OS でない Linux 版を作ることそのもの**である
- **Linux のファイルシステムは大文字小文字を区別する。**
  macOS と Windows の既定は区別しない。したがって import パスの大小のずれは
  **Linux でだけ検出できる**（`tsc` / `eslint` の検出力が上がる）
- **[ADR-0022](0022-observed-data-privacy.md) が CI に求めている検査は、すべて `tauri` に依存しない。**
  混入判定・フィクスチャのバイト一致・gitleaks・生成物の差分は、いずれも ubuntu で完結する

**Windows で確かめなければならないことは、CI では確かめられない。**

- [ADR-0029](0029-injection-ipc-transport.md) が Windows について確かめたいのは、
  **`ICoreWebView2Frame7` が実行環境に存在するか、`add_FrameCreated` が実際に
  全フレームを捕まえるか、取りこぼしが起きないか**である。
  同 ADR 自身が「未修正の公式バグが 2 件ある（最大のリスク）」「取りこぼしは黙って起きうる」と書いている
- **これはいずれも実行時の振る舞いであり、VM か実機で動かす以外に確かめる方法が無い。**
  E2E も GUI テストも存在しない
- **CI が言えるのは「型が合っている」だけ**であり、上記のどれ 1 つも保証しない。
  Windows の動作確認は Issue #6 が負っており、**CI はそれを代替しない**

- **Windows で確認する体制は現在存在しない**（Issue #6。macOS 環境で VM が要り、
  その画面はエージェントから見えない。オーナー判断で後回し）。
  したがって Windows のジョブが環境起因で落ちても、**再現もデバッグもできず、
  赤が `main` に残り続ける**

**検査を macOS / Windows で回して得られるものは、コンパイルの検査だけである。**

- **`src-tauri/src/lib.rs` は Tauri 雛形そのままの 12 行**であり、
  `#[cfg(target_os = ...)]` による分岐は **1 行も存在しない**
- OS 別コードは [ADR-0029](0029-injection-ipc-transport.md) が採ろうとしているが、
  **同 ADR は Proposed であり承認されていない。** 実装も存在しない
- **開発機は macOS であり、`task dev` / `task check` により日常的に macOS でビルドされている**

**配布用のビルドは別の話である。**

- 対応 OS は macOS / Windows であり、**配るビルドはその OS 上で作る必要がある**
  （Tauri の macOS / Windows 向けバンドルは、それぞれの OS でしか生成できない）
- **これは配布の工程であって、検査の CI ではない。**
  署名・公証・自動更新と同じ決定に属し、**本 ADR の対象外**である

## 決定

**検査のための CI を GitHub Actions に置く。ジョブは `ubuntu-latest` の 1 つだけとする。
検査に macOS / Windows は要らない。**

ワークフローに書くのは「ツールチェインを入れる」と「`task` のタスクを呼ぶ」の 2 つだけとし、
検査の中身は Taskfile と cargo / package.json 側に置いて CI 固有の記述を作らない。

- ワークフローは `.github/workflows/ci.yml` の 1 本。
  トリガーは `push`（`main`）/ `pull_request` / `workflow_dispatch`
- ジョブの内容は、**`tauri` に依存しないものすべて** ——
  型検査・リント・整形・生成物の差分・観測データの混入判定・gitleaks・
  `cargo fmt --check`・`tauri` に依存しないクレートの clippy とテスト
- **`src-tauri` のビルドは CI で行わない。**
  開発機（macOS）の `task check` に委ねる
- ツールチェインは `jdx/mise-action@v4`。Rust のビルドキャッシュに `Swatinem/rust-cache` を使う
- **観測データの混入判定（[ADR-0022](0022-observed-data-privacy.md) 具体策 3）は、
  ワークスペース内の Rust の実行可能クレートとして実装する。**
  走査対象は `git ls-files` が返す追跡対象ファイル。判定自体に単体テストを付ける。
  `tauri` に依存させないことで、ubuntu のジョブで完結させる
- **gitleaks は公式 Action を使わず、mise で CLI を入れて `task` から呼ぶ**

### 本 ADR が扱わないもの

**macOS / Windows での動作確認。**
[ADR-0029](0029-injection-ipc-transport.md) が挙げている論点
（`ICoreWebView2Frame7` の有無、`add_FrameCreated` の取りこぼし、未修正の公式バグ 2 件）は
すべて実行時の振る舞いであり、**VM か実機で動かす以外に方法が無い。**
その担保は Issue #6 が負う。**CI はそれを代替しない。**

**配布用のビルド。**
配るバイナリは対応 OS 上で作る必要があるため、**macOS と Windows でビルドする工程が要る。**
これは署名・公証・自動更新と同じ決定に属し、**本 ADR の対象外**である。

## 検討した選択肢

### 論点 1: CI を置くか

#### 案 A1: GitHub Actions に置く（採用）

- 概要: リポジトリと同じ場所で回す。public のため標準ランナーは無償・無制限
- 利点:
  - [ADR-0022](0022-observed-data-privacy.md) が「判定の正」と名指しした場所を実際に用意できる。
    ここが無い限り `.local/` を使えず、実装に着手できない
- 欠点: [ADR-0014](0014-trunk-based-on-main.md) の下では PR が無いため、
  **検出は常に push のあと**になる（下記「影響」）

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

費用は判断材料にならない（public リポジトリは無償・無制限）。
**したがって基準は「そのジョブを置くと、何が新しく分かるか」の一点である。**

#### 案 B1: `ubuntu-latest` の 1 つだけ（採用）

- 概要: `tauri` に依存しない検査をすべて ubuntu で回す。`src-tauri` は CI でビルドしない
- 利点:
  - **[ADR-0022](0022-observed-data-privacy.md) の関門を完全に満たす。**
    混入判定・フィクスチャのバイト一致・gitleaks・生成物の差分はすべて `tauri` に依存せず、
    ubuntu で完結する。**本 ADR の目的（`.local/` を使えるようにする）は 1 ジョブで達成される**
  - **Linux は大文字小文字を区別するため、静的検査の検出力がむしろ上がる**
  - 3 つの中で最も速く、**壊れたことに気づくまでの時間が短い**。
    [ADR-0014](0014-trunk-based-on-main.md) の下では検出が常に push のあとになるため、ここは効く
  - **直せない赤を抱え込まない。** ubuntu で落ちたものは手元でも再現できる
  - 振り分けの線が既存のクレート境界（`tauri` を知っているか）と一致し、迷う余地が無い
- 欠点:
  - **`src-tauri` がコンパイルできることを CI が保証しない。** 開発機の `task check` に依存する
  - 対応 OS でない Linux でだけ落ちることが起きうる（例: パスの大小）。
    ただしそれは検出力が上がった結果であり、直す価値のある誤りである

#### 案 B2: ubuntu + `windows-latest`

- 概要: `src-tauri` のビルドを Windows で行い、誰もビルドしない OS を CI で押さえる
- 利点: `src-tauri` のコンパイルを CI が保証する
- 却下理由: **得られるものが「12 行の Tauri 雛形がコンパイルできる」ことだけ**であり、
  それに対して**直せない赤を抱えるリスク**（Windows の実機も VM も無い）が釣り合わないため。
  Windows 固有コードは [ADR-0029](0029-injection-ipc-transport.md) が Proposed の段階で、
  1 行も存在しない。**存在しないコードのために CI を先に置くのは先掘りである。**
  加えて **Windows で確かめたいのは振る舞いであり、それは CI では分からず VM が要る。**
  VM で動かせばコンパイルも当然通るため、**CI は VM 実行の部分集合ですらない。**

#### 案 B3: ubuntu + `macos-latest`

- 概要: 開発機と同じ OS でビルドを確認する
- 却下理由: **macOS のビルドは開発機が毎日通しており、CI を足しても新しい情報が出ない**ため。

#### 案 B4: すべての検査を macOS + Windows の 2 つで回す（初稿の案）

- 概要: 対応 OS と CI の実行環境を完全に一致させ、ジョブを 1 種類に保つ
- 却下理由: **静的検査には OS の意味が無いのに、実行時間だけが倍になる**ため。
  加えて大文字小文字を区別しない 2 OS だけになり、Linux で回せば見つかる誤りを取り逃がす。

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
  - **`tauri` に依存しないため、ubuntu の 1 ジョブでそのまま回る**
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
- 却下理由: 判定に単体テストを付けにくいため。
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

## 決め手

**CI が確実に赤くなれる状態を守るために、「対応 OS でビルドが通ることの保証」を捨てた。**

本 CI を置く目的は、[ADR-0022](0022-observed-data-privacy.md) が名指しした
**不可逆な事故（public リポジトリの履歴に個人情報が残る。C-07）を止めること**である。
その関門は `tauri` に依存せず、ubuntu の 1 ジョブで完全に満たせる。

一方、**壊れたビルドは可逆である。** push し直せば直る。
[ADR-0022](0022-observed-data-privacy.md) の「判定の正は CI に置く」は
不可逆なものについての決定であり、**コンパイルエラーにまで同じ強度を広げる理由は無い。**

そして、**直せない赤は CI を殺す。** Windows で環境起因の失敗が起きても再現手段が無く、
「どうせ赤いから」で無視する習慣がつけば、本来止めたかった混入判定まで一緒に無視される。
**赤が必ず意味を持つ状態を保つことのほうが、ジョブの数より重い。**

**そもそも CI は「動くかどうか」を確かめる道具ではない。**
macOS / Windows で確かめたいのは振る舞いであり、**それに要るのは CI ではなく VM である**
（Issue #6）。配るバイナリを作る工程も、検査ではなく配布の決定に属する。

## 影響

### 実装への影響

- 新規に作るもの:
  - `.github/workflows/ci.yml`（`ubuntu-latest` の 1 ジョブ）
  - 観測データの混入判定を行う実行可能クレート（[ADR-0032](0032-repository-structure.md) の
    ツリーに 1 メンバー追加。**同 ADR の「強制される境界は 3 本」には影響しない**）
- `mise.toml` の `[tools]` に `gitleaks` を追加する
- **`Taskfile.yml` に足りない検査を足し、`tauri` 依存の有無で呼び分けられるように割る**:
  - 生成物の差分検査（`kcsapi-hook.js` / `bindings.ts` を再生成して差分ゼロ）。
    **`check:fe` の `desc` が「注入スクリプトの差分」と書いているのに実装されていない**穴を埋める
  - 観測データの混入判定
  - gitleaks
  - 現在の `check:rust` は `cargo clippy --workspace` であり、これは `src-tauri` を含む。
    **CI（ubuntu）用に「`tauri` に依存しないクレートだけ」の入口を別に用意する。**
    開発者が手元で叩く `task check` は、従来どおり自分の OS で全部を回す
- **[ADR-0033](0033-task-runner.md) の「`task check` は CI と同じ内容にする」は、
  本 ADR により「`task check` は CI が回す内容をすべて含み、加えて `src-tauri` も検査する」となる。**
  包含関係が変わるだけで、両者が別物になるわけではない。ADR-0033 は書き換えない
- **GitHub Actions で使う action は、版タグではなくコミットハッシュで固定する。**
  タグは付け替えられるため、タグ指定では取得する内容が変わりうる。
  人間が版を読めるよう、ハッシュの横に版を注記する
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

1. **防止（ローカル）** —— push 前に `task check` を回す。**規約であり、保証ではない**
2. **保証（CI）** —— 越えられない場所で必ず落とす。**ただし落ちた時点で既に push されている**

**1 を [CLAUDE.md](../../CLAUDE.md) §5 の完了条件に加える必要がある**（人間の承認が要る）。
`src-tauri` のビルドを CI に置かない判断は、この 1 に寄りかかっている。

### ドキュメントへの影響

- [CLAUDE.md](../../CLAUDE.md) §5「作業の完了条件」に、push 前の検査を 1 行足す（**承認が要る**）
- [CONTRIBUTING.md](../../CONTRIBUTING.md) に CI の説明と、落ちたときの直し方を足す
- `docs/spec/` の変更は無い。**CI はユーザーに見える振る舞いではない**
- `docs/guidelines/` への追記も本 ADR では行わない
  （テストとフィクスチャの規約は [ADR-0022](0022-observed-data-privacy.md) が別途 1 本を要求している）

### 取り消す場合のコスト

**低い。** ワークフローの削除・OS の増減はいつでも変えられる。
検査の中身が Taskfile 側にあるため、CI サービスを移す場合も
「ツールチェインを入れて `task` を呼ぶ」を書き直すだけで済む。

## 未解決事項

- **フィクスチャの生成器（[ADR-0022](0022-observed-data-privacy.md) 決定 3）が未実装である。**
  したがって混入判定の「生成器の出力は除外する」条件は、現時点では**除外対象ゼロ**として実装する。
  生成器を作る時点で除外の定義を埋める。
  同 ADR の未解決事項「生成器の実装形態（`cargo test` 内か独立した bin か）」も、そこで決める
- **Git フック（[ADR-0022](0022-observed-data-privacy.md) 具体策 4 の pre-commit）を本 ADR では入れない。**
  lefthook / prek などの導入は依存の追加であり、
  [ADR-0018](0018-dependencies.md) の系列で別途決める。
  **判定の正が CI にあることは変わらないため、実装着手は妨げない**
- `TODO(未確定)`: gitleaks のカスタムルールの具体的な内容。
  `api_token` を検出するルールを足すことは [ADR-0022](0022-observed-data-privacy.md) が決めているが、
  **`api_token` の実際の形式（長さ・文字種）を観測できていない**ため、
  現時点では既定ルールのみで導入し、実測後にカスタムルールを足す
- **改行の正規化（`.gitattributes`）は本 ADR では扱わない。**
  リポジトリの設定であり、CI の有無と独立に必要なものだからである
  （Windows は git の `core.autocrlf` が既定で有効であり、
  [ADR-0022](0022-observed-data-privacy.md) が決めたフィクスチャの**バイト一致検査**が
  CRLF 変換で壊れる）。設定 1 行として別に入れる
- **配布（署名・公証・自動更新）とリリースの自動化は本 ADR の対象外。**
  外部仕様と配布方法が決まってから別に起票する。
  **配るバイナリは対応 OS 上で作る必要があるため、macOS と Windows でビルドする工程は
  そこに属する**
- **macOS / Windows での動作確認（Issue #6）を CI は担保しない。** 必要なのは VM である
- CI の実行時間が問題になった場合の分割は、実際に遅くなってから判断する。
  **先に最適化しない**
