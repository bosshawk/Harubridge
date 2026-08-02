# ADR-0028: 任務カウンタのデータ形式と `count` 条件の語彙

- ステータス: **Proposed**
- 日付: 2026-08-03
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0020](0020-kancolle-reference.md)（`data/` に機械可読データを置く決定を具体化する）,
  [ADR-0021](0021-data-persistence.md)（導出状態としての任務進捗）,
  [ADR-0025](0025-clock-handling.md)（**リセット境界の時刻の求め方はそちらに従う**）,
  [FR-050 / FR-051](../spec/requirements.md),
  [C-01 / C-02 / C-03](../spec/constraints.md),
  [rules/quests.md](../kancolle/rules/quests.md),
  [api_get_member_questlist.md](../kancolle/api/api_get_member_questlist.md)

## 背景と課題

[FR-050](../spec/requirements.md) により、**任務の達成回数は自前で数える**ことが確定している。
ゲームは `api_state`（1/2/3）と `api_progress_flag`（0 / 1=50% 以上 / 2=80% 以上）しか返さず、
回数はどのフィールドにも存在しない（[questlist の記録](../kancolle/api/api_get_member_questlist.md)）。

数えるには**任務ごとの達成条件を知っている必要がある**。
[ADR-0020](0020-kancolle-reference.md) は「アプリが読むデータは `data/` に機械可読で置く」と決め、
[architecture.md](../spec/architecture.md) は「`data/` は `build.rs` で検証して埋め込む」と決めた。
しかし**中身の形は決まっていない。**

現在の [`data/kancolle/quests.json`](../../data/kancolle/quests.json) は 2 件のみの暫定であり、
`rules/quests.md` は末尾で `TODO(未確定)`「`count` の条件をどこまで表現できる語彙にするかは
実装に着手する時点で決める」と明記している。**本 ADR がそれを決める。**

### 決めなければならないこと

1. データ形式と読み込み方（JSON か。ビルド時に埋め込むか、実行時に読むか）
2. **`count` 条件を表現する語彙。どこまでをデータで書き、どこからをコードに置くか**
3. 自前カウンタの永続化と、`api_progress_flag` と食い違ったときの扱い
4. デイリー / ウィークリー / マンスリー / クォータリー / 年次のリセット

### 前提: 依存できるのは既存実装の「参照」だけである

**本プロジェクトは `api_get_member/questlist` を一度も観測していない。**
出撃・演習・遠征・工廠系のエンドポイントも**すべて未観測**である
（[api/overview.md](../kancolle/api/overview.md) の `TODO(未観測)`）。

したがって本 ADR の事実はすべて他実装のソースの参照であり、
**「どのイベントで数えるか」を実測で裏づけたものは 1 件も無い。**
この制約が決定の形そのものを決めている（後述）。

### 調査でわかった、決定に効く事実

既存 3 実装のソースを 2026-08-03 に取得して読んだ。

#### (a) 「達成条件を宣言的データで持つ」実装は存在しない

| 実装 | 任務テーブルの実体 | 条件の在り処 |
| --- | --- | --- |
| 七四式電子観測儀 | **C# のコード**（`QuestProgressManager.cs` に一意な `case <任務ID>:` が **171 個**、`new Progress*(...)` が **506 箇所**） | すべてコード |
| KC3改 | `src/data/quests_meta.json`（**776 件中 `tracking` を持つのは 135 件**。キーは `unlock` / `rewardConsumables` / `hash` / `tracking` の 4 種のみ） | **JSON にあるのは最大値だけ。**条件は `Kcsapi.js` の各 API ハンドラに `KC3QuestManager.get(606).increment()` の形で任務 ID 直書き |
| KancolleSniffer | `QuestCountList.cs`（**184 件**。C# のオブジェクト初期化子。見た目はデータだがコンパイルされる） | 大半はフィールド。**編成条件だけは別ファイルのコード** |

**本 ADR が検討している「条件を宣言的データに書く」形を採っている先行実装は無い。**
最も近い KancolleSniffer でさえ、テーブルは C# のコードである。

#### (b) 既存実装が「データ的に書ける条件」と「書けない条件」の境界を、実際に同じ場所で割っている

KancolleSniffer の `QuestSpec.cs` は、基底 `QuestSpec`
（`Interval` / `Max` / `MaxArray` / `AdjustCount` / `Shift` / `Material` / `Exp` / `Disabled`）と
派生 6 種の**閉じた型の集合**である。

| 型 | 追加フィールド | 件数 |
| --- | --- | ---: |
| `QuestSortie` | `Rank` / `Ranks` / `Maps` | 66 |
| `QuestPractice` | `Rank` | 42 |
| `QuestDestroyItem` | `Types`（装備種別の配列の配列）/ `Ids` | 30 |
| `QuestMission` | `Ids`（遠征 ID） | 18 |
| `QuestSpec`（基底のまま） | — | 13 |
| `QuestPowerUp` | `Types` / `ItemsTypes` / `Requires` | 8 |
| `QuestEnemyType` | `EnemyType`（撃沈した敵の艦種） | 8 |

**これらはすべて「値の照合」である** —— ランクの比較、ID が集合に含まれるか、真偽値。

一方、**編成条件は 1 つもここに入っていない。**
`QuestFleetChecker.cs`（**521 行**）に、**86 個の `case <任務ID>:`** として C# で書かれている。
中身は「旗艦が軽巡」「軽巡は 3 隻まで」「駆逐を含む」「他艦種は禁止」「特定艦の艦番号を 3 つ含む」
といった**述語**であり、隻数の数え上げと全称・存在量化を要する。

七四式電子観測儀も同じ場所で割れている。
17 種の `Progress*` クラスのうち `ProgressSpecialBattle` だけが **210 回**使われ、
その `Increment()` は **82 個の `case <任務ID>` を含む 27 KB の switch** である。中身は編成条件である。

> **2 実装が独立に同じ境界で割っている。**
> ただし KancolleSniffer のテーブルは七四式電子観測儀を参考に作られたと
> ソースに明記されており（[rules/quests.md](../kancolle/rules/quests.md)）、
> **この一致を「独立な 2 例」として扱ってはならない。**
> 独立なのは KC3改 だが、KC3改 は条件を一切データに持たないため、この境界に関する証言をしない。

#### (c) 「出現周期」と「進捗カウンタのリセット周期」は別物で、3 実装が食い違う

任務 337「「十八駆」演習！」は、攻略 Wiki の ID 体系では **`Cq2` = クォータリー**である。

| 実装 | 337 のリセット周期 | 根拠 |
| --- | --- | --- |
| 七四式電子観測儀 | **日次** | `GetProgressResetType()` が 311/330/337/339/341/342/348 を `QuestResetType.Daily` に上書き。`case 337` のコメントに「**クォータリーだが1日で進捗リセット**」 |
| KancolleSniffer | **日次** | `{337, new QuestPractice {Interval = Daily, …}}` |
| KC3改 | **クォータリー** | `repeatableTypes.quarterly.questIds` に 337 を含む |

**したがって、レコードが持つ周期は「任務の出現周期」ではなく
「進捗カウンタがリセットされる周期」でなければならない。**
現在の `data/kancolle/quests.json` のフィールド名 `period` はこれを区別していない。

#### (d) `api_progress_flag` は単調増加ではない

攻略 Wiki「任務」のコメント欄に、同一提督による連続した 3 件の報告がある（2026-08-03 確認）。

> 「開発資材」だけが足りず、先週末「８０％」表示で終わった。……月曜日に任務を見たら
> 「８０％」表示が消えていた（2026-07-07 投稿）
> / 開発資材の所有が100個を超えて、月曜日の任務更新で確認したら「80％」表示が復活してました（2026-07-14 投稿）
> / 開発資材の所有をわざと100個以下にしたら、「８０％」表示が消えました（2026-07-19 投稿）

**所持数を条件にする任務では、`api_progress_flag` が下がる。**
KancolleSniffer もこれに対応しており、`QuestDestroyItem` の多くに `AdjustCount = false`
（この任務ではフラグによる補正をしない）が付いている。
七四式電子観測儀にも同等の `IgnoreCheckProgress` がある。

#### (e) リセット時刻は 3 資料が一致する

| 周期 | 境界 | 出典 |
| --- | --- | --- |
| デイリー | 毎日 **05:00 JST** | KC3改「JST is +9 GMT, so 05:00 JST === 20:00 UTC」/ KancolleSniffer `LastMorning`（`Hour < 5` で前日、`AddHours(5)`）/ 攻略 Wiki「※1日(5時～翌5時)のうちに勝利7回で達成」 |
| ウィークリー | 月曜 05:00 JST | KC3改（日曜 20:00 UTC）/ KancolleSniffer `LastMonday.AddHours(5)` |
| マンスリー | 1 日 05:00 JST | KC3改 / KancolleSniffer |
| クォータリー | **3 / 6 / 9 / 12 月**の 1 日 05:00 JST | KC3改（Mar/Jun/Sep/Dec）/ KancolleSniffer（`Month / 3 * 3`） |
| 年次 | **任務ごとに決まった月**の 1 日 05:00 JST | KC3改 `yearlyJan`〜`yearlyDec` / KancolleSniffer `Yearly1`〜`Yearly12` |

年次が「1 月」ではなく**任務ごとに異なる月**であることは、両実装が独立に 12 種類の年次周期を
持っていることから確実である。**ゲームは年次任務がどの月にリセットされるかを教えてくれない**
（`api_label_type` は 102/103 で「イヤーリー」を示すだけ。参照: `apilist.txt`）。
**クォータリーに至っては `api_type` / `api_label_type` のどちらにも区分が無い。**
→ **周期はこちらのテーブルが持つしかない。**

#### (f) 起動していない間に跨いだリセットは、2 実装とも「最後にリセットした時刻」で扱う

- KancolleSniffer: `_lastReset` を永続化し、`_lastReset < boundary && boundary <= _now` で判定する
- KC3改: `timeToResetDailyQuests` などを `localStorage` に持ち、`serverTime >= resetTime` で判定する

**どちらもカウンタを「状態」として持ち、境界を跨いだら 0 に戻している。**

## 決定

### 1. 形式は JSON を継続し、`build.rs` で検証して埋め込む

[`data/kancolle/quests.json`](../../data/kancolle/quests.json) を JSON のまま維持する。
`build.rs` で `serde_json` によりパースし、下記のスキーマ検査に失敗したらビルドを落とす
（[architecture.md](../spec/architecture.md)「外部データの取り込み」）。

**実行時に読む形は採らない。** 外部から差し替える経路を作らないため（[C-04](../spec/constraints.md)）。

### 2. `count` の語彙は「値の照合」までとし、述語はデータに書かない

**これが本 ADR の中心である。**

境界を次の 1 行で定める。

> **定数との照合（等しい / 集合に含まれる / 以上）だけで書ける条件はデータに書く。
> 数え上げ・全称量化・「他は禁止」といった述語が要る条件は、データに書かない。**

`count` は「**1 個のイベント種別**」と「**そのイベントに付く値の照合フィルタ**」の組で表す。
複数条件の任務（あ号作戦、Z作戦など）は `stages` の配列にする。

```jsonc
{
  "id": 337,
  "name": "「十八駆」演習！",
  "reset": "daily",
  "counter": {
    "shift": 2,
    "adjust": true,
    "stages": [
      { "max": 3, "event": "practice_result", "rank_at_least": "S" }
    ]
  },
  "source": "…",
  "verified": null
}
```

```jsonc
{
  "id": 249,
  "name": "「第五戦隊」出撃せよ！",
  "reset": "monthly",
  "counter": null,
  "unsupported": "fleet-composition",
  "source": "…",
  "verified": null
}
```

| フィールド | 内容 |
| --- | --- |
| `id` | 任務 ID。一意 |
| `name` | **人がこのテーブルを読むための識別子**。UI は `api_title` を使い、この値を表示しない |
| `reset` | **進捗カウンタがリセットされる周期。**任務の出現周期ではない（→ 背景 (c)） |
| `counter` | `null` なら「数えない」 |
| `counter.stages[]` | 1 個以上。各段が `max` と 1 個の `event` と照合フィルタを持つ |
| `counter.shift` | 逆算モデルの上書き（[rules/quests.md](../kancolle/rules/quests.md) の採用モデル） |
| `counter.adjust` | `false` なら `api_progress_flag` による補正をしない（→ 背景 (d)） |
| `unsupported` | 語彙で表現できない理由。**`null` でなければ UI は「進捗を数えない」と表示する** |
| `source` / `verified` | 出典と確認日（[data/README.md](../../data/README.md)） |

`reset` の値は閉じた列挙とする:
`once` / `daily` / `weekly` / `monthly` / `quarterly` / `yearly-01` 〜 `yearly-12` / `unknown`。

`unsupported` の値も閉じた列挙とする:
`fleet-composition`（編成条件）/ `possession`（所持数の判定）/ `shared-counter`（他任務と共有）/
`unobserved-event`（数える契機が未観測）/ `unknown`。

### 3. 語彙は Rust 側の enum を正とし、`data/` はそれに無い値を書けない

**`event` とフィルタの名前の集合は、Rust の enum が正である。**
`build.rs` は enum に無い値を見つけたらビルドを落とす。

これにより「**表現できるのに検証されていない条件**」が原理的に発生しない。
語彙を 1 つ増やすには、実装を 1 つ増やす必要がある。

**本 ADR では `event` の具体的な値を確定しない。**
数える契機となるエンドポイントを**本プロジェクトは 1 つも観測していない**ためである。
初期の候補は既存 3 実装が購読しているものから採るが、
**実測して構造を確認したものから 1 つずつ enum に追加する**（[C-03](../spec/constraints.md)）。

`TODO(要検証)` 実装着手時に、下記を実測してから語彙を確定する。

| 候補となる契機 | 参照元 | 実測 |
| --- | --- | --- |
| 出撃の戦闘結果（ランク / 海域 / ボスか） | 3 実装すべてが `api_req_sortie/battleresult` 等を購読 | **未観測** |
| 演習の結果（ランク） | 同上 `api_req_practice/battle_result` | **未観測** |
| 遠征の帰還（遠征 ID / 成否） | 同上 `api_req_mission/result` | **未観測** |
| 撃沈した敵の艦種 | 七四式 `ProgressSlaughter` / Sniffer `QuestEnemyType` | **未観測** |
| 工廠（開発・建造・解体・廃棄・改修・近代化改修） | 3 実装すべて | **未観測** |
| 入渠・補給 | 3 実装すべて | **未観測** |

### 4. カウンタは「状態」ではなく「リセット境界以降の事象の集計」と定義する

[ADR-0021](0021-data-persistence.md) が **`events/*.jsonl` を正、`state/quests.json` をキャッシュ**と
定めたことにより、既存 3 実装ができなかった形が採れる。

> **表示値 = max( 直近のリセット境界以降の `events/` を数えた値, `floor` )**

- `floor` は `api_progress_flag` から逆算した**下限**であり、`state/quests.json` に持つ
- `floor` もリセット境界で捨てる
- `counter.adjust` が `false` の任務では `floor` を更新しない（→ 背景 (d)）
- **`floor` は下げない。** `api_progress_flag` が下がっても `floor` は据え置く

**これによりリセットの取りこぼしが原理的に起きない。**
アプリを起動していない間に境界を跨いでも、集計の下限が動くだけである。
「最後にリセットした時刻」を持つ必要がない（→ 背景 (f) の 2 実装と異なる）。

`state/quests.json` を失っても `events/` から再集計でき、失うのは `floor` だけである。
`floor` は次にプレイヤーが任務画面を開いた時点で取り直せる。

境界の時刻の求め方は [ADR-0025](0025-clock-handling.md) に従う
（**JST 固定オフセット (+09:00)。端末のローカル時刻を経由しない**）。
本 ADR はそれを前提とし、独自には決めない。
なお KancolleSniffer は `DateTime.Now`（端末ローカル）で 5 時を判定しており、
日本国外では境界がずれる。この点は ADR-0025 が既に同じ指摘をしている。

### 5. FR-051（いつ時点の値か）は 3 つの時刻で満たす

`state/quests.json` の各任務に次を持つ。

| 値 | 意味 |
| --- | --- |
| `counted_through` | この時刻までの `events/` を数えた |
| `flag_observed_at` | 最後に `questlist` でこの任務の `api_progress_flag` を見た時刻 |
| `floor_estimated` | 表示値が `floor` 由来（＝逆算した推定値）であるか |

**`questlist` はプレイヤーが任務画面を開いたときにしか飛ばず、しかもタブ単位である。**
こちらから取りに行くことは [C-01](../spec/constraints.md) / [C-02](../spec/constraints.md) に反する。
したがって `flag_observed_at` が古いままになることは避けられず、
**それをユーザーに見せることが FR-051 の実体である。**

`TODO(未確定)`: この 3 つをユーザーにどう見せるかは**外部仕様の領分**であり、
本 ADR では決めない（人間の承認が要る。[CLAUDE.md](../../CLAUDE.md) §2）。

## 検討した選択肢

### 案 A: 値の照合までの閉じた語彙 + 「表現できないものは載せない」（採用）

- 概要: 上記のとおり。述語が要る任務は `counter: null` + `unsupported` を書き、
  UI は「進捗を数えない」と表示して縮退する
- 利点:
  - 評価器が「照合」で済み、インタプリタにならない
  - 語彙が Rust の enum に閉じているため、**未実装の条件をデータに書けない**
  - 既存実装が実際に割っている境界と一致しており、
    「ここから先は述語になる」という判断が経験的に裏づけられている（→ 背景 (b)）
  - 未登録・非対応の任務は `rules/quests.md` が既に決めている縮退（「進捗を数えない」と表示）に
    そのまま乗る
- 欠点:
  - **マンスリー・クォータリーの主力任務がまとめて非対応になる。**
    KancolleSniffer が編成条件を要するとしている 86 件は、原則としてすべて `unsupported` になる。
    249 /257 /259 /264 /266 /280 /284 といった、ユーザーが最も進捗を見たい任務が含まれる
  - 「編成条件をコードに書く」逃げ道を塞いでいるため、後で必要になったら本 ADR を覆すことになる

### 案 B: 述語をデータに書ける語彙にする（条件の DSL）

- 概要: `{"all": [{"flagship_type": "CL"}, {"count": {"type": "CL"}, "max": 3}, …]}` のような
  入れ子の条件式をデータに書き、評価器を実装する
- 利点: 編成条件を含むほぼ全任務をデータで表現でき、任務追加でコードを触らずに済む
- 欠点: 評価器が小さなインタプリタになる。**組み合わせの大半はどの任務からも使われず、
  テストもされない。** それでいてデータ側からは書けてしまう
- 却下理由: **語彙を広げるほど「表現できるのに検証されていない条件」が増える**という、
  本 ADR がまさに避けたい状態を作る。しかも先行実装が 1 つも無く、
  難易度の見積もりが立たない（→ 背景 (a)）。

### 案 C: 条件をすべて Rust のコードに書く（七四式電子観測儀の方式）

- 概要: `data/kancolle/quests.json` を廃し、任務 ID の巨大な `match` を Rust に書く
- 利点: 表現力に上限が無い。実際に 3 実装のうち 2 つがこの形に落ち着いている
- 欠点: 出典（`source` / `verified`）をレコードに持たせられない。
  `build.rs` での検証（[architecture.md](../spec/architecture.md)）が効く対象が無くなる
- 却下理由: [ADR-0020](0020-kancolle-reference.md) の
  「アプリが読むデータは `data/` に置き、出典を各レコードに持たせる」を覆すことになる。
  艦これ側の観測結果は**ランタイムの選択より長生きする**という同 ADR の前提が失われる。

### 案 D: 最大値と周期だけをデータに持ち、条件は API ハンドラに直書きする（KC3改 の方式）

- 概要: `quests.json` は `{"id": 606, "max": 1, "reset": "daily"}` 程度に留め、
  各 API のハンドラで任務 ID を直に加算する
- 利点: 最も単純。KC3改 が実際にこれで 135 件を運用している
- 却下理由: 任務を 1 件足すのに触る場所が 3 か所（メタ / ハンドラ / 表示）に散る。
  実際 KC3改 の `Kcsapi.js` には `KC3QuestManager.get(606).increment()` の形で
  任務 ID が直接埋め込まれており、**どの任務がどのイベントで数えられるかを
  一覧できる場所が存在しない。** 出典の併記もできない。

### 案 E: 形式を TOML にする

- 概要: `data/kancolle/quests.toml` にし、`toml` クレートで読む
- 利点: コメントが書ける（JSON では `source` を書く欄が要る理由が消える）。
  行志向のため git の差分が読みやすい。`toml` は crates.io 実測 90 日 1 億 8373 万 DL（2026-08-03）で
  健全
- 却下理由: `counter.stages[]` が**配列の中のテーブルの中の配列**になり、
  TOML では `[[quests.counter.stages]]` の見出しが縦に伸びて 1 任務が読みづらくなる。
  実データの入れ子の深さ（KancolleSniffer の `Types = [[1], [2], [5, 32]]` に相当するもの）が
  TOML の得意な形から外れている。**行志向の利点が、この形では出ない。**

### 案 F: 形式を YAML にする

- 却下理由: `serde_yaml` は **`0.9.34+deprecated` が最新安定版で、最終更新が 2024-03-25**
  （crates.io、2026-08-03 実測）。作者が保守を終了している。
  後継の `serde_yaml_ng` も最終更新 2024-05-26 / 90 日 414 万 DL と規模が 2 桁小さい。
  **[ADR-0018](0018-dependencies.md) が採った「実測の DL 数と更新状況で選ぶ」基準を満たさない。**

### 案 G: 形式を RON にする

- 却下理由: Rust の enum をタグ付きで自然に書ける利点はあるが、
  90 日 2438 万 DL（`serde_json` の 1/10 以下、2026-08-03 実測）で、
  **Rust の外から編集・検証する道具が事実上無い。**
  任務テーブルは今後 10 年単位で人が手で足していくデータであり、
  ランタイムに紐づかない形式であることを優先する（[data/README.md](../../data/README.md) の理由と同じ）。

### 案 H: 実行時にデータファイルを読む

- 却下理由: 手で書くデータであり必ず誤りが混入する。
  **実行時に読むと、誤りが「ユーザーの環境で初めて」露見する。**
  [architecture.md](../spec/architecture.md) が既に「ビルド時に検出する」と定めており、
  それを覆す理由が見つからなかった。

### 案 I: `api_progress_flag` に合わせて自前カウンタを双方向に補正する（KC3改 の方式）

- 概要: 自前カウンタから逆算した段階が実際の段階と食い違ったら、**下げる方向にも**書き換える
- 却下理由: **`api_progress_flag` は単調増加ではない**（→ 背景 (d)）。
  所持数が条件に入る任務では、装備を使っただけで 80% 表示が消える。
  そこで自前カウンタを下げると、**数えた事実（`events/` にある）を推測で上書きする**ことになる。
  [ADR-0021](0021-data-persistence.md) の「事象が正」に反する。

### 案 J: 進捗をユーザーが手で修正できるようにする

- 却下理由: **ユーザーに見える振る舞いであり、外部仕様が無い。**
  [FR-050](../spec/requirements.md) / FR-051 のどちらも要求していない。
  仕様の追加には人間の承認が要る（[CLAUDE.md](../../CLAUDE.md) §2）ため、本 ADR では決めない。
  必要と判断されたら外部仕様に起こしてから実装する。

## 決め手

**表現力を捨てて、「表現できるのに検証されていない条件が存在しない」状態を買った。**

任務の条件は本質的に述語であり、真面目に表現しようとすれば必ずミニ言語になる。
だが本プロジェクトは**数える契機そのものを 1 つも観測していない**。
検証できないものを書けるようにすることは、[C-03](../spec/constraints.md) が禁じている
「推測を確定仕様として扱う」ことそのものである。

## 影響

- 実装への影響:
  - `build.rs` に任務テーブルの検証を追加する。`event` / `reset` / `unsupported` の値は
    Rust の enum に照合し、未知の値でビルドを落とす
  - `serde_json` を Rust の依存に追加する（[ADR-0018](0018-dependencies.md) の一覧に 1 行）
  - 任務の進捗は「集計 + 下限」の 2 値で持つ。単一のカウンタとして実装しない
  - **編成条件を要する任務は初回リリースで数えられない。** UI は縮退表示を必ず持つ
- データへの影響:
  - [`data/kancolle/quests.json`](../../data/kancolle/quests.json) は**本 ADR が承認されるまで変更しない**
    （`Proposed` は実装の根拠にしてはいけない。[README.md](README.md)）。
    承認後、現行 2 件を新スキーマに移す。`period` → `reset` の改名を含む
- ドキュメントへの影響:
  - [rules/quests.md](../kancolle/rules/quests.md) の末尾 `TODO(未確定)`
    「`count` の条件をどこまで表現できる語彙にするか」が解消される（**別途更新が要る**）
  - 「アプリ導入前の進捗は復元できない」「表示が古いままになりうる」は
    [ADR-0021](0021-data-persistence.md) が既に**外部仕様が要る**としている。本 ADR も同じ結論
  - [architecture.md](../spec/architecture.md) への追記は不要（既存の記述の範囲内）
- 取り消す場合のコスト: **低〜中。**
  語彙を広げる方向（案 B へ）は、既存レコードを壊さずに追加できる。
  逆に `data/` を捨ててコードに戻す（案 C）場合は、出典の持ち方から作り直しになる。

## 未解決事項

- `TODO(要検証)`: **`event` の具体的な値。**
  数える契機となるエンドポイントを 1 つも観測していないため確定できない。
  出撃・演習・遠征・工廠を含む 1 セッションを観測すれば埋まる。
  [ADR-0021](0021-data-persistence.md) が「最優先」としている観測と同一のものである
- `TODO(要検証)`: **`api_get_member/questlist` 自体が未観測。**
  `api_progress_flag` の実際の値と実際の達成回数の対応が確認できていない以上、
  `floor` の逆算モデル（`shift`）の正しさは検証されていない
  （[rules/quests.md](../kancolle/rules/quests.md) も同じ `TODO` を持つ）
- `TODO(要検証)`: **リセット境界がサーバ時刻とクライアント時刻のどちらで決まるか。**
  [rules/timers.md](../kancolle/rules/timers.md) は「PC の時計がサーバとずれていると
  完了時刻の表示が前後する」と記録しているが、任務のリセットについては確認していない。
  [ADR-0025](0025-clock-handling.md) は「時計のずれは補正せず、警告も出さない」と決めており、
  本 ADR もそれに従う。**ただしリセット境界について実測した資料は無い**
- `TODO(要検証)`: **共有カウンタの扱い。**
  デイリーの開発・建造など、複数任務が同じ回数を共有する事例が知られている
  （[rules/quests.md](../kancolle/rules/quests.md)）。
  七四式電子観測儀は `SharedCounterShift` というフィールドで補正しているが、
  本 ADR のスキーマには対応する概念が無い。`unsupported: "shared-counter"` で逃がすか、
  フィールドを足すかは実測してから決める
- `TODO(未確定)`: **`unsupported` の任務をユーザーにどう見せるか。**外部仕様の領分
- `TODO(未確定)`: **編成条件を要する任務を、どこまで諦めるか。**
  本 ADR は「データに書かない」と決めたが、
  「一部の任務だけコードに書く」道を将来選ぶ可能性は残っている。
  その判断は、実際に何件が非対応になるかを数えてから行う
- `TODO(未確定)`: 任務テーブルを増やしていく作業の進め方（どの任務から埋めるか）。
  [rules/quests.md](../kancolle/rules/quests.md) は「よく使う任務から埋める」とだけ決めている

## 参照した資料

| 記述 | 出典 | ライセンス | 参照日 |
| --- | --- | --- | --- |
| `QuestSpec` の基底と派生 6 種、184 件のテーブル、`QuestFleetChecker.cs` の 521 行 / 86 case、`QuestCounter.cs` の任務 ID 直書き、リセット境界の実装、`_lastReset` の永続化 | bitbucket.org/kancollesniffer/kancollesniffer `KancolleSniffer/Model/QuestSpec.cs` / `QuestCountList.cs` / `QuestCounter.cs` / `QuestFleetChecker.cs` / `QuestInfo.cs` | **Apache-2.0**（`Copyright (C) 2013-2021 Kazuhiro Fujieda` / `Copyright (C) 2021 hATrayflood`）。**参照のみ。コードは流用しない**（[sources.md](../kancolle/sources.md) C-6） | 2026-08-03 |
| 171 の一意な `case`、506 箇所の `new Progress*`、17 種の `Progress*`、`ProgressSpecialBattle` の 82 case / 27 KB、`CheckProgress` / `IsOldQuest` / `SharedCounterShift` / `IgnoreCheckProgress`、337 の「クォータリーだが1日で進捗リセット」 | github.com/ElectronicObserverEN/ElectronicObserver `ElectronicObserver/Data/Quest/QuestProgressManager.cs` / `ProgressData.cs` / `ProgressSpecialBattle.cs` | MIT（`Original work copyright (c) 2014 Andante` / `Translation work copyright (c) 2015 Ryuu Kitsune`） | 2026-08-03 |
| `quests_meta.json` の 776 件中 135 件が `tracking`、キーが 4 種のみ、`repeatableTypes` のリセット時刻と questIds、`Kcsapi.js` の任務 ID 直書き | github.com/KC3Kai/KC3Kai `src/data/quests_meta.json` / `src/library/managers/QuestManager.js` / `src/library/modules/Kcsapi.js` | MIT | 2026-08-03 |
| デイリー境界「1日(5時～翌5時)」、定期任務の ID 体系（Bd/Bw/Bm/Bq/By ほか）、`api_progress_flag` が下がる報告 | [艦これ攻略 Wiki「任務」](https://wikiwiki.jp/kancolle/%E4%BB%BB%E5%8B%99) | 明示なし。**参照のみ、転載不可** | 2026-08-03 |
| `serde_json` 1.0.151 / 90 日 2 億 5607 万、`toml` 1.1.4 / 1 億 8373 万、`serde_yaml` 0.9.34+deprecated（最終更新 2024-03-25）、`serde_yaml_ng` 0.10.0 / 414 万、`ron` 0.12.2 / 2438 万 | crates.io API | — | 2026-08-03 |

> `kcwikizh/kcdata` は**ライセンス表記が無く参照も流用も不可**であるため、
> 任務データの取得先として検討していない（[sources.md](../kancolle/sources.md) F-1）。
> 本 ADR はどの実装からもデータそのものを取り込んでいない。**決めたのはスキーマだけである。**
