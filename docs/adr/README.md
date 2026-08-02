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
| `Accepted（…のみ Superseded by …）` | 決定の一部だけが置き換えられた。残りは有効 |
| `Superseded by ADR-NNNN` | 全体が別の ADR に統合・置換された。本文は履歴として残る |

## ADR にする条件

**次の 3 つをすべて満たすものだけ**を ADR にする。1 つでも欠けたら書かない。

1. **選択肢を実際に比較した。** 却下案とその理由が事実として書ける
2. **覆すコストが高い。** 公開済み / 外部依存 / データ形式 / 法的なもの
3. **事情を知らない人が「なぜ？」と再提案しうる**

該当する典型例:

- **技術的な判断** —— 言語・フレームワーク・実行形態・ライブラリの選定、
  データの持ち方と保存場所、外部依存の追加、通信の観測方式、配布方法。
  **このプロジェクトで ADR の中心になるのはここである。**
- 法的・規約的なリスクに関わる方針（ライセンス、権利物の扱い）
- 文書とコードの役割分担のような、全体に効く方針

## ADR にしないもの

| 内容 | 正しい置き場所 |
| --- | --- |
| ユーザーの指示・方針をそのまま記録するだけのもの | `CLAUDE.md` / `docs/` に**規範として**書く |
| まだ動いている検討の途中経過 | **GitHub Issue**。固まってから 1 本書く |
| 1 行の設定変更、局所的で簡単に戻せる判断 | 該当する文書を直すだけ |
| 「いまどうなっているか」の説明 | `docs/spec/` / `docs/README.md` |

### 守ること

- **却下案を捏造しない。** 実際に比較していない案を、テンプレートを埋めるために
  書いてはいけない。却下理由が事実として書けないなら、それは ADR にする段階にない。
- **同じ日のうちに何度も動いている論点を ADR にしない。** 落ち着いてから 1 本書く。
- **迷ったら書かない。** 書かなかった決定は Issue と git 履歴に残る。
  一方、質の低い ADR は将来のエージェントに誤った前提を与える。

> 本プロジェクトは初期に ADR を書きすぎ、構成に関する 6 本が
> [ADR-0015](0015-documentation-layout.md) に統合された。上記はその反省による基準である。

## 一覧

| No. | タイトル | ステータス | 日付 |
| --- | --- | --- | --- |
| [0001](0001-record-architecture-decisions.md) | 意思決定を ADR として記録する | Accepted | 2026-08-01 |
| [0002](0002-documentation-structure.md) | ドキュメントを notes / spec / adr の 3 層で管理する | Superseded by ADR-0015 | 2026-08-01 |
| [0003](0003-agent-driven-development.md) | 実装はすべて AI エージェントが行う | Accepted | 2026-08-01 |
| [0004](0004-defer-tech-stack-decision.md) | 技術構成の決定を要求定義のあとに延期する | Superseded by ADR-0016 | 2026-08-01 |
| [0005](0005-publish-as-oss.md) | OSS として公開する | Accepted | 2026-08-01 |
| [0006](0006-split-external-and-internal-spec.md) | 仕様を外部仕様・内部仕様・ガイドラインに分割する | Superseded by ADR-0015 | 2026-08-01 |
| [0007](0007-observability-based-spec-boundary.md) | 外部仕様と内部仕様の切り分けを可観測性で判断する | Superseded by ADR-0015 | 2026-08-02 |
| [0008](0008-code-as-source-of-truth.md) | 詳細仕様はコードを正とし、文書は基本設計とガイドラインに絞る | Accepted | 2026-08-02 |
| [0009](0009-notes-as-github-issues.md) | その場の方針・検討メモを GitHub Issue に移す | Superseded by ADR-0015 | 2026-08-02 |
| [0010](0010-top-level-doc-layout.md) | ドキュメントの最上位構成を目的別の 3 ディレクトリに整理する | Superseded by ADR-0015 | 2026-08-02 |
| [0011](0011-license-mit.md) | ライセンスに MIT License を採用する | Accepted | 2026-08-02 |
| [0012](0012-local-notes-directory.md) | `docs/notes/` をローカル専用のメモ置き場として残す | Superseded by ADR-0015 | 2026-08-02 |
| [0013](0013-copyright-holder.md) | 著作権表記を `bosshawk` とする | Accepted | 2026-08-02 |
| [0014](0014-trunk-based-on-main.md) | 当分の間 `main` に直接コミットする | Accepted | 2026-08-02 |
| [0015](0015-documentation-layout.md) | ドキュメント構成を確定する（0002/0006/0007/0009/0010/0012 を統合） | Accepted | 2026-08-02 |
| [0016](0016-tech-stack.md) | 技術構成に Tauri + Rust + React を採用する | Accepted | 2026-08-02 |
| [0017](0017-llm-endpoint.md) | LLM 連携は OpenAI 互換エンドポイント 1 本に絞る | Accepted | 2026-08-02 |
| [0018](0018-dependencies.md) | 依存ライブラリを選定する | Accepted | 2026-08-02 |

> 新しい ADR を追加したら、この一覧にも 1 行足すこと。
