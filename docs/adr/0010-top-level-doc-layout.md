# ADR-0010: ドキュメントの最上位構成を目的別の 3 ディレクトリに整理する

- ステータス: Accepted
- 日付: 2026-08-02
- 決定者: プロジェクトオーナー
- 関連: [ADR-0002](0002-documentation-structure.md), [ADR-0006](0006-split-external-and-internal-spec.md)（`guidelines/` の配置を変更する）, [ADR-0008](0008-code-as-source-of-truth.md)

## 背景と課題

[ADR-0006](0006-split-external-and-internal-spec.md) でガイドラインを
`docs/spec/guidelines/` に置いたが、これは構造上の必然ではなく、
既存の `docs/guides/`（開発の進め方）との**名前衝突を親ディレクトリで回避した**結果だった。

実際には、ガイドラインは仕様の一部ではない。

| | `spec/external/` | ガイドライン |
| --- | --- | --- |
| 対象 | この機能 | 全コード |
| 寿命 | 機能が消えれば消える | プロジェクトと同じ |
| 変わる要因 | 要求が変わったとき | 技術構成が変わったとき |

また `docs/guides/` は `agent-workflow.md` 1 ファイルのみで、
その内容の約半分は CLAUDE.md やスキル定義への参照・重複だった。
1 ファイルのためにディレクトリを 1 階層設ける必要はない。

加えて `guides` と `guidelines` は 3 文字違いで、
親ディレクトリが違うだけでは読み手（特にエージェント）が混同する。

## 決定

最上位構成を、**それぞれ別の問いに答える 3 ディレクトリ + 2 ファイル**に整理する。

```
CLAUDE.md         エージェントが従う規約（自動で読まれる）
CONTRIBUTING.md   進め方・役割分担・依頼の書き方（人が読む）
docs/
├── adr/          なぜそう決めたか
├── spec/         何を作るか（要求 + 基本設計 + 外部仕様）
└── guidelines/   どう書くか（横断規約）
```

1. **`docs/spec/guidelines/` を `docs/guidelines/` へ移す。**
   仕様と並列にし、`guides` との名前衝突を解消する。
2. **`docs/guides/` を解体し、[CONTRIBUTING.md](../../CONTRIBUTING.md) にする。**
   OSS として公開する（[ADR-0005](0005-publish-as-oss.md)）以上、
   進め方の置き場所は GitHub が Issue / PR 画面で案内する慣行の場所に従う。
   独自のディレクトリを発明しない。
3. **`architecture.md` は `docs/spec/` に残す。**
   日本語の慣行では 要件定義 → 基本設計（外部設計）→ 詳細設計 と並び、
   基本設計と外部仕様は同じ層にある。「プロダクトの定義は `spec/` を見ればよい」を保つ。

境界: **CONTRIBUTING.md は規約を再掲せず、CLAUDE.md にリンクする。**
CLAUDE.md はエージェントが従う規範、CONTRIBUTING.md は人が読む進め方。

## 検討した選択肢

### 案 A: 3 ディレクトリ + CONTRIBUTING.md（採用）

- 利点: 各ディレクトリが別の問い（なぜ / なに / どう書く）に対応し、
  名前だけで置き場所が決まる。`guides` / `guidelines` の混同が消える。
  進め方はエコシステムの慣行に乗り、外部の読み手にも見つかる。
- 欠点: `docs/` の外にファイルが 2 つ増える。

### 案 B: `docs/process/` を新設して `guides/` をリネームする

- 概要: 進め方を `docs/process/agent-workflow.md` に置く。
- 却下理由: 1 ファイルのためのディレクトリである点は変わらない。
  また OSS では CONTRIBUTING.md が探される場所であり、独自名は発見されにくい。

### 案 C: 現状維持（`spec/guidelines/` と `guides/` を併存）

- 却下理由: 名前の紛らわしさを親ディレクトリで誤魔化しており、
  読み手が毎回どちらか確認する必要がある。

## 決め手

**ディレクトリ数の少なさを得るために、分類の網羅性を捨てた。**
「進め方」を独立した階層として持つのをやめ、慣行のファイル 1 枚に畳んだ。

## 影響

- ドキュメントへの影響:
  - `docs/spec/guidelines/` → `docs/guidelines/`
  - `docs/guides/agent-workflow.md` → `CONTRIBUTING.md`（重複部分を削除して再構成）
  - `README.md` / `CLAUDE.md` / `docs/README.md` / `docs/spec/README.md` /
    各スキルのリンクを張り替える。
- 実装への影響: なし。
- 取り消す場合のコスト: 低（ファイル移動とリンク修正のみ）。

## 未解決事項

なし。
