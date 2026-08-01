# Harubridge

艦隊これくしょん（艦これ）向けの**専用ブラウザ（専ブラ）**を開発するプロジェクトです。

> **現在のフェーズ: 設計準備**
> 技術構成（言語 / フレームワーク / 実行形態）は**まだ決定していません**。
> 本リポジトリはまずドキュメント基盤から立ち上げています。実装コードはまだありません。

---

## このプロジェクトの前提

- **プログラムはすべて AI エージェント（Claude Code / Codex など）が作成します。**
  人間が担うのは「要求を出す」「意思決定する」「レビューして受け入れる」の 3 つです。
- したがって **ドキュメントがエージェントへの唯一の入力**であり、仕様書の曖昧さ・古さは
  そのまま実装の欠陥になります。「コードを直す前にドキュメントを直す」が原則です。
- エージェントが従うべき規約は [CLAUDE.md](CLAUDE.md) に集約しています。

## ドキュメントの置き場所

| 場所 | 何を書くか | 性質 |
| --- | --- | --- |
| [`README.md`](README.md) | プロジェクトの入口。全体像とリンク集 | 低頻度更新 |
| [`CLAUDE.md`](CLAUDE.md) | エージェントが常に従う作業規約 | 低頻度更新 |
| [`docs/spec/`](docs/spec/) | **いま何を作るのか**（確定仕様・現在形で書く） | 上書き更新 |
| ├ [`external/`](docs/spec/external/) | 外部仕様: ユーザーから見える振る舞い | 承認が必要 |
| ├ [`internal/`](docs/spec/internal/) | 内部仕様: それをどう実現するか | エージェントが更新可 |
| └ [`guidelines/`](docs/spec/guidelines/) | ガイドライン: 機能に属さない横断ルール | 承認が必要 |
| [`docs/adr/`](docs/adr/) | **なぜそう決めたのか**（意思決定の記録） | 追記のみ・不変 |
| [`docs/notes/`](docs/notes/) | 暫定メモ・調査ログ・使い捨ての下書き | 高頻度・破棄前提 |
| [`docs/guides/`](docs/guides/) | 開発の進め方・手順書 | 低頻度更新 |
| [`.claude/skills/`](.claude/skills/) | Claude Code 用スキル（作業手順の型） | 低頻度更新 |

判断に迷ったときの原則は 3 つだけです。

1. **決めたこと**（そして捨てた選択肢）は ADR へ。あとから書き換えない。
2. **今の正解**は spec へ。過去の経緯は書かず、常に最新状態だけを書く。
3. **まだ決まっていないこと**は notes へ。決まったら spec / ADR へ昇格させて notes は畳む。

詳しい運用ルールは [docs/README.md](docs/README.md) を参照してください。

## まず読むもの

- [docs/spec/overview.md](docs/spec/overview.md) — 何を作ろうとしているか
- [docs/spec/constraints.md](docs/spec/constraints.md) — 前提と制約（**実装前に必読**）
- [docs/adr/](docs/adr/) — これまでの意思決定
- [docs/guides/agent-workflow.md](docs/guides/agent-workflow.md) — エージェント駆動開発の回し方

## 現在の状態

| 項目 | 状態 |
| --- | --- |
| ドキュメント基盤 | 構築済み |
| 要求仕様 | ドラフト（[docs/spec/requirements.md](docs/spec/requirements.md)） |
| 公開方針 | OSS 公開（[ADR-0005](docs/adr/0005-publish-as-oss.md)）。ライセンスは未選定 |
| 技術構成 | **未決定**（論点整理中: [docs/notes/2026-08-01-tech-stack-open-questions.md](docs/notes/2026-08-01-tech-stack-open-questions.md)） |
| 実装 | 未着手 |

## 名前について

`Harubridge` はプロジェクトのコードネームです。アプリケーションの正式名称は
別途決定し、決まり次第 ADR に記録します。

## ライセンス / 免責

- 本プロジェクトは非公式のサードパーティ製ツールであり、DMM.com、株式会社 C2 プレパラート、
  その他「艦隊これくしょん」の権利者とは一切関係ありません。
- 本プロジェクトは **OSS として公開します**（[ADR-0005](docs/adr/0005-publish-as-oss.md)）。
  **ライセンスは未選定です。** 決定時に ADR を起票し `LICENSE` を追加します。
  それまでは、法的には作者の著作権が留保された状態であることにご留意ください。
- 利用にあたっての制約・方針は [docs/spec/constraints.md](docs/spec/constraints.md) に定義しています。
  特に、本アプリは**ゲームプレイの自動化を行わず、通信を読み取り専用として扱います**。
