# 仕様書 (spec)

**いま何を作るのか**を現在形で書く場所。ここに書かれているものが実装の根拠になる。

## 構成

```
docs/spec/
├── overview.md / requirements.md / constraints.md / glossary.md   要求層: 何が必要か
├── external/     外部仕様: ユーザーから見える振る舞い
├── internal/     内部仕様: それをどう実現するか
└── guidelines/   ガイドライン: 個別機能に属さない横断ルール
```

分割の理由は [ADR-0006](../adr/0006-split-external-and-internal-spec.md)。

### 要求層

| ファイル | 内容 |
| --- | --- |
| [`overview.md`](overview.md) | 何を作るか、対象ユーザー、スコープと非スコープ |
| [`constraints.md`](constraints.md) | 前提と制約。**実装前に必ず読む** |
| [`requirements.md`](requirements.md) | 機能要求 (FR) / 非機能要求 (NFR) の一覧 |
| [`glossary.md`](glossary.md) | 用語集（艦これ用語 + プロジェクト用語） |

### 設計層

| ディレクトリ | 内容 | 変更するとき |
| --- | --- | --- |
| [`external/`](external/) | 画面・操作・表示・設定・通知 | **人間の承認が必要** |
| [`internal/`](internal/) | モジュール構成・データモデル・処理フロー | 外部仕様を変えない限りエージェントが更新してよい |
| [`guidelines/`](guidelines/) | コーディング規約・エラー処理・テスト方針など | **人間の承認が必要** |

`internal/` と `guidelines/` は、技術構成が決まるまで原則として空
（[ADR-0004](../adr/0004-defer-tech-stack-decision.md)）。

## 読む順番

1. [`overview.md`](overview.md) — 何を作るか
2. [`constraints.md`](constraints.md) — 越えてはならない制約
3. [`requirements.md`](requirements.md) — 要求一覧
4. [`external/`](external/) — 対象機能の外部仕様
5. [`internal/`](internal/) — 対象機能の内部仕様
6. [`guidelines/`](guidelines/) — 書き方の規約

## 書き方のルール

- **現在形で書く。** 「〜する」「〜である」。「〜する予定」「〜だった」は書かない。
- **経緯を書かない。** なぜそう決めたかは [ADR](../adr/) にリンクする。
- **要求には ID を振る。** `FR-nnn` / `NFR-nnn` 形式。一度振った ID は再利用しない
  （削除した要求の ID は欠番のままにする）。
- **未確定を隠さない。** `TODO(未確定): …` として明示する。空欄・省略は禁止。
- **出典の無い事実を書かない。** 特に艦これの API 仕様は公式に公開されていないため、
  実測ログや参照 OSS を根拠として併記する。根拠が無ければ `TODO(要検証)`。
- **同じ事実を外部仕様と内部仕様の両方に書かない。** 片方が必ず古くなる。リンクする。

## 新しい機能仕様を追加するとき

1. [`external/_template.md`](external/_template.md) を `external/<機能名>.md` にコピーして書く
2. 技術構成が決まっていれば、[`internal/_template.md`](internal/_template.md) を
   `internal/<機能名>.md` にコピーして書く（ファイル名を外部仕様と揃える）
3. [`requirements.md`](requirements.md) の該当 FR から相互リンクを張る
4. 各ディレクトリの README の一覧表に 1 行追加する

外部仕様と内部仕様の切り分けに迷ったら:
**「実装を全部書き直しても、この記述は変わらないか？」**
変わらないなら外部仕様、変わるなら内部仕様。
