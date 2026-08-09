# ADR-0033: 開発コマンドの入口として Taskfile を採用する

- ステータス: **Proposed**
- 日付: 2026-08-09
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0031](0031-toolchain-management.md)（**本 ADR がタスクランナーの項のみを置き換える。
  mise / rustup / pnpm の決定は有効**）, [ADR-0032](0032-repository-structure.md),
  [ADR-0003](0003-agent-driven-development.md)

## 背景と課題

[ADR-0031](0031-toolchain-management.md) は「タスクランナー（just / make 等）を入れない。
npm scripts と cargo の直叩きで足りる」と決めた。根拠は
①デファクトが存在しない ②`make` は Windows に無い ③直叩きで足りる、の 3 つだった。

骨格の作成（[ADR-0032](0032-repository-structure.md)、2026-08-09）で**③の前提が変わった。**

- 検査一式が **2 ツールチェイン・7 コマンド**に跨がった
  （`tsc` ×2 / `eslint` / `prettier` / `clippy` / `cargo fmt` / `cargo test`）
- 「全部チェック」の入口が無く、エージェントも CI も同じ列挙を繰り返すことになる
- ADR-0031 は「npm scripts に環境変数や `rm -rf` を書くと Windows で壊れる。
  クロスプラットフォームな記述に限る規約が要る」を未解決のまま残していた

①は今も真である（本 ADR もデファクトとは主張しない）。②はむしろ選定基準になる。

### 調査した事実（2026-08-09 実測）

| | go-task/task | casey/just |
| --- | --- | --- |
| star | 15,943 | 35,197 |
| 最終 push | 2026-08-08（活発） | 2026-08-07（活発） |
| mise registry | あり（`task`） | あり（`just`） |
| Windows | **sh 解釈系を内蔵**（下記） | 別途シェルが必要 |

Taskfile 公式ドキュメントの明記:

> Task uses **mvdan/sh, a native Go sh interpreter**. So you can write sh/bash-like commands -
> **even in environments where `sh` or `bash` are usually not available (like Windows)**.

動作検証: `mise install` で task 3.52.0 を導入し、`task check`（FE + Rust の全検査）と
`task test` が通ることを確認済み。

## 決定

**開発コマンドの入口として Taskfile（go-task）を採用する。版は mise で固定する
（[ADR-0031](0031-toolchain-management.md) の機構にそのまま乗る）。**

- ルートに `Taskfile.yml` を置く。**書くのは「集約」だけ**とし、
  個別のコマンド定義は package.json / cargo に置いたまま重複させない
- 入口は `task dev` / `task build` / `task check` / `task test` / `task fmt`。
  `task check` は CI と同じ内容にする
- **ADR-0031 のタスクランナーの項（「入れない」）のみを置き換える。**
  mise / rustup / pnpm / `.nvmrc` を作らない等、他の決定はすべて有効のまま

## 検討した選択肢

### 案 A: Taskfile / go-task（採用）

- 利点: **sh 解釈系を内蔵し、Windows でシェルの前提が要らない。**
  ADR-0031 の未解決 TODO（クロスプラットフォームなスクリプト記述の規約）が
  規約ではなくツールの性質として解決する。mise で版を固定できる。記法が YAML で
  エージェントの誤記が起きにくい
- 欠点: star は just の半分以下。タスクランナー自体のデファクトは依然として存在しない

### 案 B: just

- 概要: star 35,197 と最大勢力。Spacedrive が採用
- 却下理由: Windows で外部シェル（sh 等）の導入が前提になり、
  対応 OS（macOS / Windows）の要件に対して Taskfile より一段弱いため。

### 案 C: make

- 却下理由: Windows に無い（[ADR-0031](0031-toolchain-management.md) で却下済み。再掲）。

### 案 D: mise tasks（mise 同梱のタスクランナー）

- 概要: 追加ツールなしで `mise.toml` の `[tasks]` に書ける。jj が採用
- 却下理由: jj 自身が「Mise support is experimental」と明記する段階であり
  （Issue #7 の調査）、タスクランナーとしての採用実績が Taskfile より薄いため。
  ツールチェイン管理（mise）とコマンド定義（Taskfile）を分けておけば、
  どちらかを乗り換えるときに他方が巻き込まれない利点もある

### 案 E: 現状維持（npm scripts + cargo 直叩き）

- 概要: ADR-0031 の決定
- 却下理由: 検査が 2 ツールチェイン・7 コマンドに跨がった時点で「全部チェック」の
  入口が無く、エージェントと CI が同じ列挙を重複して持つことになるため（背景のとおり）。

## 決め手

**「クロスプラットフォームな記述規約を書いて守らせる」問題を、
シェルを内蔵するツールを選ぶことで問題ごと消した。**
規約は破られうるが、内蔵シェルは迂回できない（[ADR-0003](0003-agent-driven-development.md) の
体制では、人が守る規約より構造で守るほうが強い —— [ADR-0032](0032-repository-structure.md) と同じ論法）。

## 影響

- `mise.toml` に `task = "3.52.0"` が加わる（パッチまで固定。ADR-0031 の方針どおり）
- ルートに `Taskfile.yml` が加わる
- [ADR-0031](0031-toolchain-management.md) のステータスを
  「Accepted（タスクランナーの項のみ Superseded by ADR-0033）」に変更する
- README の開発手順が `task --list` を起点にできる
- CI は `task check` を呼ぶ形にできる（CI 自体は未整備のまま）
- 取り消す場合のコスト: **低。** `Taskfile.yml` と mise の 1 行を消せば
  元の直叩きに戻る（コマンド定義は package.json / cargo に残っている）

## 未解決事項

- `TODO(未確定)`: CI の構成。`task check` を呼ぶ方針だけが決まった
- `TODO(未確定)`: 生成物の差分ゼロ検査（`bindings.ts` / `kcsapi-hook.js`）を
  `task check` に含めるか。CI 整備時に判断する
