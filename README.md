# Harubridge

艦隊これくしょん（艦これ）向けの**専用ブラウザ（専ブラ）**を開発するプロジェクトです。

> **現在のフェーズ: 要求定義**
> 技術構成は決定済み（[Tauri + Rust + React](docs/adr/0016-tech-stack.md)）。
> 要求は TBD で、実装コードはまだありません。

---

## このプロジェクトの前提

- **プログラムはすべて AI エージェント（Claude Code / Codex など）が作成します。**
  人間が担うのは「要求を出す」「意思決定する」「レビューして受け入れる」の 3 つです。
- したがって **ユーザーに見える振る舞いは、必ず先にドキュメントで決めます**。
  仕様書の曖昧さ・古さはそのまま実装の欠陥になるためです。
- 一方で **実装の詳細はコードを正とし、文書化しません**
  （[ADR-0008](docs/adr/0008-code-as-source-of-truth.md)）。二重管理された文書は必ず古くなり、
  古い文書はエージェントにとって誤った前提として作用するためです。
- エージェントが従うべき規約は [CLAUDE.md](CLAUDE.md) に集約しています。

## ドキュメントの置き場所

| 場所 | 何を書くか | 性質 |
| --- | --- | --- |
| [`README.md`](README.md) | プロジェクトの入口。全体像とリンク集 | 低頻度更新 |
| [`CLAUDE.md`](CLAUDE.md) | エージェントが常に従う作業規約 | 低頻度更新 |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | 開発の進め方・役割分担・依頼の書き方 | 低頻度更新 |
| [`docs/spec/`](docs/spec/) | **いま何を作るのか**（確定仕様・現在形で書く） | 上書き更新 |
| ├ [`architecture.md`](docs/spec/architecture.md) | 基本設計: 機能をまたいで効く構造 | 維持・メンテ |
| └ [`external/`](docs/spec/external/) | 外部仕様: ユーザーへの約束と受け入れ条件 | 最小限 |
| **コード** | 個別機能の実現方法 | **文書化しない** |
| **GitHub Issue** | その場の方針・検討・調査 | 閉じたら終わり |
| [`docs/notes/`](docs/notes/) | 手元の書き捨てメモ | **git 管理外・共有されない** |
| [`docs/guidelines/`](docs/guidelines/) | **どう書くか**（横断規約） | 維持・メンテ |
| [`docs/adr/`](docs/adr/) | **なぜそう決めたのか**（意思決定の記録） | 追記のみ・不変 |
| [`.claude/skills/`](.claude/skills/) | Claude Code 用スキル（作業手順の型） | 低頻度更新 |

判断に迷ったときの原則は 3 つだけです。

1. **決めたこと**（そして捨てた選択肢）は ADR へ。あとから書き換えない。
2. **今の正解**は spec へ。ただし**コードを読めば分かることは書かない**。
3. **まだ決まっていないこと**は GitHub Issue へ。決まったら ADR にして閉じる。
   他人が読む必要のない書き捨てメモだけ `docs/notes/`（共有されません）。

詳しい運用ルールは [docs/README.md](docs/README.md) を参照してください。

## まず読むもの

- [docs/spec/overview.md](docs/spec/overview.md) — 何を作ろうとしているか
- [docs/spec/constraints.md](docs/spec/constraints.md) — 前提と制約（**実装前に必読**）
- [docs/adr/](docs/adr/) — これまでの意思決定
- [CONTRIBUTING.md](CONTRIBUTING.md) — 開発の回し方（役割分担・依頼の書き方）

## 現在の状態

| 項目 | 状態 |
| --- | --- |
| ドキュメント基盤 | 構築済み |
| 要求仕様 | **TBD**（[docs/spec/requirements.md](docs/spec/requirements.md)。別途相談して作成） |
| 公開方針 | OSS 公開（[ADR-0005](docs/adr/0005-publish-as-oss.md)） / [MIT License](LICENSE) |
| 技術構成 | **Tauri + Rust + React**（[ADR-0016](docs/adr/0016-tech-stack.md)）。対応は macOS / Windows |
| 実装 | 未着手 |

## 名前について

`Harubridge` はプロジェクトのコードネームです。アプリケーションの正式名称は
別途決定し、決まり次第 ADR に記録します。

## ライセンス / 免責

- 本プロジェクトは非公式のサードパーティ製ツールであり、DMM.com、株式会社 C2 プレパラート、
  その他「艦隊これくしょん」の権利者とは一切関係ありません。
- 本プロジェクトは **OSS として公開します**（[ADR-0005](docs/adr/0005-publish-as-oss.md)）。
  ライセンスは **[MIT License](LICENSE)** です（[ADR-0011](docs/adr/0011-license-mit.md)）。
  このライセンスは本リポジトリのコードとドキュメントにのみ及び、
  ゲーム側の権利物には一切関係しません。
- 利用にあたっての制約・方針は [docs/spec/constraints.md](docs/spec/constraints.md) に定義しています。
  特に、本アプリは**ゲームプレイの自動化を行わず、通信を読み取り専用として扱います**。
