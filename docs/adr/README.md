# Architecture Decision Records (ADR)

このプロジェクトの**意思決定の記録**。何を決めたかと同じくらい、
**何を捨てたか・なぜ捨てたか**を残すことを目的とする。

## ルール

- 1 決定 = 1 ファイル。ファイル名は `NNNN-english-kebab-title.md`（連番、欠番禁止）。
- テンプレート: [`template.md`](template.md) をコピーして書く。
- **一度 `Accepted` になった ADR の本文は書き換えない。**
  決定を変えるときは新しい ADR を起票し、旧 ADR のステータス行だけを
  `Superseded by ADR-NNNN` に更新する。
- 誤字修正や、決定内容に影響しない補足は可。

## ステータス

| ステータス | 意味 |
| --- | --- |
| `Proposed` | 提案中。まだ実装の根拠にしてはいけない |
| `Accepted` | 採用。実装はこれに従う |
| `Superseded by ADR-NNNN` | 別の ADR に置き換えられた |
| `Deprecated` | 前提が消滅し、置き換え先も無い |
| `Rejected` | 検討したが採用しなかった（記録として残す） |

## ADR を書くべき判断

- あとから変えるコストが高いもの（言語 / フレームワーク / 実行形態 / 配布方法）
- 外部への依存を増やすもの
- データの持ち方・保存場所を決めるもの
- 法的・規約的なリスクに関わる方針
- チーム（人間 + エージェント）の作業ルールを変えるもの

逆に、**局所的で簡単に戻せる判断は ADR にしない**。コード内コメントか spec で足りる。

## 一覧

| No. | タイトル | ステータス | 日付 |
| --- | --- | --- | --- |
| [0001](0001-record-architecture-decisions.md) | 意思決定を ADR として記録する | Accepted | 2026-08-01 |
| [0002](0002-documentation-structure.md) | ドキュメントを notes / spec / adr の 3 層で管理する | Accepted | 2026-08-01 |
| [0003](0003-agent-driven-development.md) | 実装はすべて AI エージェントが行う | Accepted | 2026-08-01 |
| [0004](0004-defer-tech-stack-decision.md) | 技術構成の決定を要求定義のあとに延期する | Accepted | 2026-08-01 |

> 新しい ADR を追加したら、この一覧にも 1 行足すこと。
