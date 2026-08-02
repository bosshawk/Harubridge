# ドキュメント運用ガイド

このリポジトリのドキュメントを「どこに・どう書き・いつ捨てるか」のルール。
判断の背景は [ADR-0015](adr/0015-documentation-layout.md)（構成）と
[ADR-0008](adr/0008-code-as-source-of-truth.md)（何を書かないか）。

## 1. 何を残し、何を流すか

**残すのは、コードから読み取れず、古くなると誤解を生むものだけ。**
それ以外は流す（Issue）か、書かない（コード）。

```
docs/notes/    手元の書き捨てメモ         → git 管理外・共有されない
   │ 共有すべきなら
GitHub Issue   その場の方針・検討・調査   → 閉じたら終わり
   │ 決まったら
   ├──→ docs/adr/    決定「なぜそう決めたか」        → 追記のみ・不変
   ├──→ docs/spec/   仕様「いまどうなっているか」    → 維持・メンテ
   └──→ コード       実装の詳細                      → 文書化しない
```

| | Issue | spec | adr | コード |
| --- | --- | --- | --- | --- |
| 内容 | 検討中の論点 | 約束・構造・規約 | 決定と却下案 | 実現方法 |
| 時制 | 疑問形 | **現在形** | 決定時点の記録 | — |
| 更新 | 閉じて終わり | 常に最新へ上書き | **書き換えない**（状態欄のみ） | 随時 |
| 寿命 | 決着まで | プロジェクトと同じ | 永久 | — |

### どこに書くかの判断

```
その情報は「選択」か？（他の案を捨てて 1 つを選んだ）
  ├ YES → ADR を起票する
  └ NO  → まだ決まっていないことか？
            ├ YES → GitHub Issue を立てる
            └ NO  → コードを読めば分かることか？
                      ├ YES → 書かない（コードが正）
                      └ NO  → spec を更新する
```

最後の分岐の詳細は [spec/README.md](spec/README.md#文書とコードの境界) を正とする。

## 2. spec（仕様書）

`docs/spec/` 配下。**常に「現在の正解」だけ**が書かれている状態を保つ。

- 経緯・却下案・議論の跡を残さない。それらは ADR / Issue の仕事。
- 要求には ID を振る（`FR-nnn` / `NFR-nnn`）。エージェントへの実装依頼と
  テストがこの ID で仕様を参照できるようにするため。
  **要求そのものは現在 TBD**（[spec/requirements.md](spec/requirements.md)）。
- 未確定部分は消さずに `TODO(未確定): …` として残す。**黙って空欄にしない。**
- **迷ったら書かない。** 書いた分だけメンテ義務が増える
  （[ADR-0008](adr/0008-code-as-source-of-truth.md)）。

構成（詳細は [spec/README.md](spec/README.md)）:

| 場所 | 内容 | 扱い |
| --- | --- | --- |
| [`overview.md`](spec/overview.md) | 何を作るか・スコープと非スコープ | 維持 |
| [`constraints.md`](spec/constraints.md) | 前提と制約。**実装前に必ず読む** | 維持 |
| [`requirements.md`](spec/requirements.md) | 機能要求 (FR) / 非機能要求 (NFR) | 維持 |
| [`glossary.md`](spec/glossary.md) | 用語集（艦これ用語 + 本プロジェクト用語） | 維持 |
| [`architecture.md`](spec/architecture.md) | **基本設計**: 機能をまたいで効く構造 | **維持・メンテ** |
| [`external/`](spec/external/) | 外部仕様: ユーザーへの約束と受け入れ条件 | 最小限 |

ガイドラインは仕様ではないため、[`docs/guidelines/`](guidelines/) に並列で置く
（[ADR-0015](adr/0015-documentation-layout.md)）。

**機能ごとの内部仕様は存在しない。コードが正**
（[ADR-0008](adr/0008-code-as-source-of-truth.md)）。
処理フロー・クラス構成・データ構造は文書化しない。

### 変更時の承認境界

（[ADR-0015](adr/0015-documentation-layout.md)）

| 場所 | 変更するとき |
| --- | --- |
| 要求層（overview / requirements / constraints） | **人間の承認が必要** |
| `spec/architecture.md` | 構造を変える変更は**承認が必要** |
| `spec/external/` | **人間の承認が必要**（ユーザーに見える振る舞いが変わる） |
| `guidelines/` | **人間の承認が必要**（影響が全体に及ぶ） |
| コード | 外部仕様と基本設計を変えない限り、エージェントの判断で書いてよい |

ただし**相互リンクの追記と一覧表の同期は承認不要**。ここで作業を止めない。

何を文書にし何をコードに委ねるかの判断基準は
**[spec/README.md](spec/README.md#文書とコードの境界) の 1 箇所を正とする**
（[ADR-0015](adr/0015-documentation-layout.md) /
[ADR-0008](adr/0008-code-as-source-of-truth.md)）。ここには再掲しない。

## 2b. kancolle（艦これ側の仕様）

[`docs/kancolle/`](kancolle/) 配下。**外部システムの観測結果**を記録する
（[ADR-0020](adr/0020-kancolle-reference.md)）。

- `api/` — `/kcsapi/` のエンドポイントとレスポンスの構造
- `formulas/` — 計算式（制空値・索敵値など）
- `rules/` — ゲームのルール（枠数・回復時間など）

**「コードが正」は自分たちの実装についてのルールであり、ここには当てはまらない。**
コードには実装した範囲しか現れず、観測して分かったことのほうが広い。
**出典と観測日を必ず書く**（[C-03](spec/constraints.md)）。

## 3. adr（Architecture Decision Record）

`docs/adr/` 配下。形式は [MADR](https://adr.github.io/madr/) の簡略版。
詳細は [adr/README.md](adr/README.md)。

- ファイル名: `NNNN-english-kebab-title.md`（連番は欠番を作らない）
- ステータス: `Proposed` → `Accepted` → （必要なら）`Superseded by ADR-NNNN` / `Deprecated`
- **決定を覆すときは新しい ADR を書く。** 古い ADR の本文は歴史として残す。
- **ADR にする条件は 3 つすべてを満たすものだけ**（[adr/README.md](adr/README.md#adr-にする条件)）。
  選択肢を実際に比較した / 覆すコストが高い / 事情を知らない人が再提案しうる。
  技術構成・外部依存・データの持ち方・配布形態といった**技術的な判断が中心**になる。
- **却下案を捏造しない。** 比較していない案をテンプレートを埋めるために書かない。

## 4. その場の方針・検討メモ（GitHub Issue）

**リポジトリ内には置かない。** GitHub Issue を使う
（[ADR-0015](adr/0015-documentation-layout.md)）。

- 論点が生じたら `gh issue create`。ラベルで種類を分ける
  （`question` / `research` / `decision`）。
- 決着したら **ADR を起票し、Issue に ADR へのリンクを貼って閉じる。**
  Issue の本文を仕様書に転記しない。
- エージェントは `gh issue list` / `gh issue view` で読む。
- 手順は `.claude/skills/capture-issue/SKILL.md`。

### 手元の作業メモ

書き捨てのメモは [`docs/notes/`](notes/) に置く。**git 管理対象外であり共有されない**
（[ADR-0015](adr/0015-documentation-layout.md)）。

判断基準はひとつ。**他人（将来の自分を含む）が読む必要があるか。**
あるなら Issue、無いなら `docs/notes/`。迷ったら Issue。

## 5. guidelines（横断規約）

[`docs/guidelines/`](guidelines/) 配下。コーディング規約・エラー処理方針・テスト方針など、
**個別機能に属さないルール**。仕様（何を作るか）とは別の軸なので並列に置く。

開発の**進め方**（誰が何をするか、依頼の書き方）は
[CONTRIBUTING.md](../CONTRIBUTING.md) にある（[ADR-0015](adr/0015-documentation-layout.md)）。

## 6. 変更時の同期義務

| 変更したもの | 一緒に更新するもの |
| --- | --- |
| ユーザーに見える振る舞い | `docs/spec/external/` |
| 機能をまたぐ構造 | `docs/spec/architecture.md` |
| **上記に該当しない実装** | **何も更新しない**（コードが正） |
| spec の重要な方針 | 根拠となる ADR（無ければ起票） |
| ADR を Accepted にした | spec への反映、元になった Issue のクローズ |
| 用語を新しく使い始めた | `glossary.md` |

この同期が取れているかは `.claude/skills/docs-audit` で点検できる。

## 7. 書き方の約束

- 日本語。ですます調ではなく**である調**で簡潔に。
- 1 文 1 主張。長い前置きを書かない。
- 断定できないことは断定しない。`TODO(要検証)` / `TODO(未確定)` を使う。
- 外部の記述を根拠にするときは URL と参照日を書く。
- 図が要るときは Mermaid を使う（外部画像に依存しない）。
