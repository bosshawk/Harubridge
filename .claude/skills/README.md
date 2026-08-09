# Claude Code スキル

反復する作業の型を定義したもの。Claude Code が該当する作業を検知して自動的に読み込む。
明示的に呼び出すこともできる。

各スキルは `<name>/SKILL.md` に置き、YAML フロントマターで `name` と `description` を持つ。
`description` に「いつ使うか」が書かれているかで、自動選択の精度が決まる。

## 一覧

| スキル | 用途 |
| --- | --- |
| [`write-adr`](write-adr/SKILL.md) | 設計判断を ADR として記録する |
| [`write-spec`](write-spec/SKILL.md) | `docs/spec/` の仕様書を書く・更新する |
| [`capture-issue`](capture-issue/SKILL.md) | 論点を GitHub Issue に残す・ADR 化して閉じる |
| [`research-kancolle`](research-kancolle/SKILL.md) | 艦これ側の仕様を調べて `docs/kancolle/` に記録する |
| [`validate-data`](validate-data/SKILL.md) | `data/` 配下のデータの出典・整合性を検証する |
| [`docs-audit`](docs-audit/SKILL.md) | ドキュメントの整合性・鮮度を点検する |
| [`audit-adr-refs`](audit-adr-refs/SKILL.md) | `docs/adr/` の外に ADR への参照が無いか点検する |

## サブエージェント

`.claude/agents/` にエージェント定義を置く。スキルが「手順」であるのに対し、
エージェントは「その手順を独立した文脈で実行する担当」である。

| エージェント | 用途 |
| --- | --- |
| `kancolle-researcher` | 艦これの仕様調査。`research-kancolle` スキルに従う |
| `kancolle-oss-analyst` | 既存の艦これ OSS の内部実装の調査。ライセンス規律を組み込み |
| `kancolle-expert` | 艦これ仕様の相談役。`docs/kancolle/` を拠り所に確度付きで答える |
| `data-validator` | `data/` の検証。`validate-data` スキルに従う |
| `docs-auditor` | 仕様・文書の整合性点検。`docs-audit` / `audit-adr-refs` スキルに従う |
| `rust-implementer` | Rust コア（`crates/` / `src-tauri/`）の実装。G-RS ルールを遵守 |
| `frontend-implementer` | TypeScript / React（`src/`）と注入スクリプトの実装。G-TS ルールを遵守 |
| `rust-reviewer` | Rust コードのレビュー。修正せず、ルール ID 付きの指摘を返す |
| `typescript-reviewer` | TypeScript / React コードのレビュー。修正せず、ルール ID 付きの指摘を返す |

## スキルを追加するとき

- 「1 回きりの作業」はスキルにしない。**2 回以上繰り返す手順**だけを型にする。
- `description` は「何をするか」だけでなく **「どういう場面で使うか」** を書く。
- 手順はチェックリスト形式にする。エージェントが抜けを自己検出できるようにするため。
- 追加したらこの一覧と [CONTRIBUTING.md](../../CONTRIBUTING.md) を更新する。

## 他のエージェント（Codex など）との関係

`.claude/` は Claude Code 固有の設定。ツール非依存の規約は
[`CLAUDE.md`](../../CLAUDE.md) と [`docs/`](../../docs/) にある。
**同じ内容を二重管理しない。** 他のエージェント用の設定が必要になった場合も、
`CLAUDE.md` を参照させる形にする。
