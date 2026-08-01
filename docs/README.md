# ドキュメント運用ガイド

このリポジトリのドキュメントを「どこに・どう書き・いつ捨てるか」のルール。
判断の背景は [ADR-0002](adr/0002-documentation-structure.md) にある。

## 1. 3 層モデル

ドキュメントは**変化の速さ**で 3 層に分ける。層をまたいで同じことを書かない。

```
docs/notes/   暫定    「まだ分からない / 検討中」   → 使い捨て、破棄前提
   │ 決まったら昇格
   ├─────────────→ docs/adr/    決定    「なぜそう決めたか」  → 追記のみ・不変
   └─────────────→ docs/spec/   仕様    「いまどうなっているか」→ 上書き更新
```

| | notes | spec | adr |
| --- | --- | --- | --- |
| 時制 | 過去形・疑問形 | **現在形**（〜する / 〜である） | 決定時点の記録 |
| 更新 | 自由。放置せず畳む | 常に最新へ上書き | **書き換えない**（状態欄のみ変更可） |
| 読者 | 書いた本人と数日後の自分 | 実装するエージェント | 将来の意思決定者 |
| 寿命 | 数日〜数週間 | プロジェクトと同じ | 永久 |

### どこに書くかの判断

```
その情報は「選択」か？（他の案を捨てて 1 つを選んだ）
  ├ YES → ADR を起票する
  └ NO  → それは「実装が従うべき決まりごと」か？
            ├ YES → spec を更新する
            └ NO  → notes に書く
```

## 2. spec（仕様書）

`docs/spec/` 配下。**常に「現在の正解」だけ**が書かれている状態を保つ。

- 経緯・却下案・議論の跡を残さない。それらは ADR / notes の仕事。
- 要求には ID を振る（`FR-001`, `NFR-001`）。エージェントへの実装依頼と
  テストがこの ID で仕様を参照できるようにするため。
- 未確定部分は消さずに `TODO(未確定): …` として残す。**黙って空欄にしない。**
- **同じ事実を外部仕様と内部仕様の両方に書かない。** 片方が必ず古くなる。リンクする。

構成（詳細は [spec/README.md](spec/README.md)）:

| 場所 | 内容 |
| --- | --- |
| [`overview.md`](spec/overview.md) | 何を作るか・対象ユーザー・スコープと非スコープ |
| [`constraints.md`](spec/constraints.md) | 前提と制約。**実装前に必ず読む** |
| [`requirements.md`](spec/requirements.md) | 機能要求 (FR) / 非機能要求 (NFR) の一覧 |
| [`glossary.md`](spec/glossary.md) | 用語集（艦これ用語 + 本プロジェクト用語） |
| [`external/`](spec/external/) | 外部仕様: ユーザーから見える振る舞い |
| [`internal/`](spec/internal/) | 内部仕様: それをどう実現するか |
| [`guidelines/`](spec/guidelines/) | ガイドライン: 個別機能に属さない横断ルール |

### 変更時の承認境界

仕様の中でも、変更したときの重さは一様ではない
（[ADR-0006](adr/0006-split-external-and-internal-spec.md)）。

| 場所 | 変更するとき |
| --- | --- |
| 要求層（overview / requirements / constraints） | **人間の承認が必要** |
| `external/` | **人間の承認が必要**（ユーザーに見える振る舞いが変わる） |
| `internal/` | 外部仕様を変えない限り、エージェントの判断で更新してよい |
| `guidelines/` | **人間の承認が必要**（影響が全体に及ぶ） |

外部仕様と内部仕様の切り分け基準は
**[spec/README.md](spec/README.md#外部仕様と内部仕様の切り分け) の 1 箇所を正とする**
（[ADR-0007](adr/0007-observability-based-spec-boundary.md)）。ここには再掲しない。

## 3. adr（Architecture Decision Record）

`docs/adr/` 配下。形式は [MADR](https://adr.github.io/madr/) の簡略版。
詳細は [adr/README.md](adr/README.md)。

- ファイル名: `NNNN-english-kebab-title.md`（連番は欠番を作らない）
- ステータス: `Proposed` → `Accepted` → （必要なら）`Superseded by ADR-NNNN` / `Deprecated`
- **決定を覆すときは新しい ADR を書く。** 古い ADR の本文は歴史として残す。
- 「技術構成」「外部依存」「データの持ち方」「配布形態」など、
  **あとから変えるのが高くつく判断**は必ず ADR にする。

## 4. notes（暫定メモ）

`docs/notes/` 配下。**捨てる前提**で気軽に書く場所。

- ファイル名: `YYYY-MM-DD-english-kebab-topic.md`
- 冒頭に必ずヘッダを置く（[`_template.md`](notes/_template.md) 参照）:

  ```markdown
  - 状態: 進行中 | 昇格済み | 破棄
  - 昇格先: docs/adr/0005-xxx.md（昇格済みの場合）
  ```

- **昇格ルール**: メモの内容が確定したら
  - 選択 → ADR を起票 → メモを `昇格済み` にして `docs/notes/archive/` へ移動
  - 決まりごと → spec に反映 → 同上
  - 不要になった → `破棄` にして archive へ、または削除
- 進行中のメモが 1 か月以上放置されたら、棚卸しして畳む（`docs-audit` スキル）。

## 5. guides（手順書）

`docs/guides/` 配下。「どう進めるか」の話。仕様（何を作るか）とは分ける。

[`docs/spec/guidelines/`](spec/guidelines/) と紛らわしいので区別すること。

- `docs/guides/` = **開発の進め方**（プロセス・手順・誰が何をするか）
- `docs/spec/guidelines/` = **成果物のルール**（コードや UI が従うべき規約）

## 6. 変更時の同期義務

| 変更したもの | 一緒に更新するもの |
| --- | --- |
| 実装 | 対応する spec |
| spec の重要な方針 | 根拠となる ADR（無ければ起票） |
| ADR を Accepted にした | spec への反映、元になった notes の昇格処理 |
| 用語を新しく使い始めた | `glossary.md` |

この同期が取れているかは `.claude/skills/docs-audit` で点検できる。

## 7. 書き方の約束

- 日本語。ですます調ではなく**である調**で簡潔に。
- 1 文 1 主張。長い前置きを書かない。
- 断定できないことは断定しない。`TODO(要検証)` / `TODO(未確定)` を使う。
- 外部の記述を根拠にするときは URL と参照日を書く。
- 図が要るときは Mermaid を使う（外部画像に依存しない）。
