# API: `api_get_member/questlist` — 任務一覧と進捗

- パス: `/kcsapi/api_get_member/questlist`
- 最終観測: 2026-08-02
- 確度: **本文書に実測はひとつも無い。全て参照である**（理由は[下記](#本文書に実測が無い理由)）

## 結論 —— ゲームは「何回中何回」を返さない

**進捗は、ゲーム側から「達成回数 / 最大値」の形では届かない。**
レスポンスに含まれるのは次の 2 つだけである（参照）。

| 何が届くか | フィールド | 値 |
| --- | --- | --- |
| 受注状態 | `api_state` | 1=未受領 / 2=遂行中 / 3=達成 |
| **粗い進捗段階** | `api_progress_flag` | 0=50% 未満（達成済も 0）/ 1=**50% 以上** / 2=**80% 以上** |

**「3/5」のような数値は、どのフィールドにも存在しない。**
`apilist.txt`（フィールドの意味）・kcsapi.ts（TypeScript 型）・
ElectronicObserverEN（C# 型）・実装 3 本のいずれにも、回数を表すフィールドは無い。

したがって、既存ツールが表示している「達成回数/最大値」は**ツールが自前で数えた値**である。
本調査で確認した 3 実装（七四式電子観測儀 / KC3改 / KancolleSniffer）は、いずれも

1. **自前のカウンタ**（現在値・最大値）を持ち、
2. **各操作の API を契機に自分で加算し**、
3. `api_progress_flag` は**自前カウンタのズレを補正する用途にしか使っていない**

という同じ構造をとっている。詳細は[既存ツールの実装](#既存ツールがどう数えているか参照)。

> **`api_progress_flag` は進捗の「値」ではなく「区間の下限」である。**
> 1 が返ってきたときに分かるのは「50% 以上 80% 未満」だけで、
> そこから逆算できる回数はひとつに定まらない。
> 実際、[max=3 の任務では実装ごとに逆算結果が食い違っている](#食い違い-max3-の任務の逆算結果)。

## 本文書に実測が無い理由

本プロジェクトの実測（2026-08-02、[overview.md](overview.md) の 1 セッション）では、
**`api_get_member/questlist` を 1 度も観測していない。**
そのセッションで任務画面を開いたかどうかは記録に残っていない。

- `TODO(要検証)`: 任務画面を開いたときに本当にこのエンドポイントが飛ぶか（実測）
- `TODO(要検証)`: `api_progress_flag` の実際の値と、そのときの実際の達成回数の対応（実測）

**したがって本文書の記述はすべて他資料の参照であり、確度は「参照」である。**

## いつ飛ぶか

**プレイヤーが任務画面を開いたとき**（参照）。母港に戻るたびには飛ばない。

- リクエストに `api_tab_id`（開いたタブ）が含まれる。
  **タブごとの取得であり、1 回のレスポンスに全任務が入るとは限らない**（参照: `apilist.txt`）
- KC3改 のコードには「**2020-03-27 以降、`api_disp_page` は無くなり、
  `api_tab_id` で指定された種別の項目が全件返る**（ゲーム内 UI 上のページ送りは残っている）」
  というコメントがある（参照: `Kcsapi.js`）。
  ただし型定義側には `api_disp_page` / `api_page_count` が省略可能フィールドとして残っている

## リクエスト

| 項目 | 内容 |
| --- | --- |
| メソッド | `TODO(要検証)`（他エンドポイントと同じく POST と推定されるが未確認） |
| 主なパラメータ | `api_tab_id` / `api_page_no`（省略可）/ `api_verno` |

`api_tab_id` の値（参照: `apilist.txt`）:

| 値 | タブ |
| --- | --- |
| 0 | すべて |
| 1 | デイリー |
| 2 | ウィークリー |
| 3 | マンスリー |
| 4 | 単発 |
| 5 | 他 |
| 9 | 遂行中 |

**認証パラメータの値は書かない**（[C-04](../../spec/constraints.md)）。

## レスポンス

- 形式: `svdata=` + JSON（[overview.md](overview.md) の封筒構造に従う）
- 観測したサイズ: **未観測**

### `api_data` 直下

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_count` | number | 任務数 | 参照 |
| `api_exec_count` | number | **受領中の任務数** | 参照 |
| `api_exec_type` | number | 不明 | 参照（出典側も「？」としている） |
| `api_completed_kind` | number | 大淀のボイスフラグ（0=通常, 1=任務達成時） | 参照 |
| `api_list` | array \| null | 任務の配列。**要素が `-1`（数値）のことがある**。0 件のときは `null` | 参照 |
| `api_c_list` | array | 省略可。`api_list` の外側にある別配列 | 参照 |
| `api_disp_page` | number | 省略可。表示ページ | 参照 |
| `api_page_count` | number | 省略可。総ページ数 | 参照 |

> `api_list` の要素は「任務オブジェクト」か「数値 `-1`」のどちらかである。
> kcsapi.ts が `(APIList | number)[] | null`、
> ElectronicObserverEN が `List<object>` に
> 「要素の型は `ApiListClass` または `int`」というコメントを付けている。
> KancolleSniffer は `if (entry is double) continue;`、
> KC3改 は `if(!questList[ctr] || questList[ctr] === -1) continue;` で読み飛ばしている。
> **4 資料が独立に同じ扱いをしており、この挙動は確度が高い。**

### `api_list[]` の要素

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_no` | number | 任務 ID | 参照 |
| `api_state` | number | **1=未受領 / 2=遂行中 / 3=達成** | 参照 |
| **`api_progress_flag`** | number | **0 / 1=50% 以上 / 2=80% 以上。達成済でも 0 になる** | 参照 |
| `api_category` | number | 分類（1=編成, 2=出撃, 3=演習, 4=遠征, 5=補給/入渠, 6=工廠, 7=改装, 8/9=出撃(2)(3)） | 参照 |
| `api_type` | number | 出現タイプ（1=デイリー, 2=ウィークリー, 3=マンスリー, 4=単発, 5=他） | 参照 |
| `api_label_type` | number | 省略可。周期アイコン種別（1=単発, 2=デイリー, 3=ウィークリー, 6=マンスリー, 7=他, 102/103=イヤーリー） | 参照 |
| `api_title` | string | タイトル | 参照 |
| `api_detail` | string | 説明。改行は `<br>` | 参照 |
| `api_get_material` | number[] | 達成時の獲得資源（4 要素） | 参照 |
| `api_bonus_flag` | number | 1=通常 / 2=艦娘 | 参照 |
| `api_select_rewards` | array[][] | 省略可。選択報酬 | 参照 |
| `api_lost_badges` | number | 省略可。消費勲章数 | 参照 |
| `api_voice_id` | number | 達成時ボイス ID | 参照 |
| `api_invalid_flag` | number | 機種転換不能フラグ（0=可能, 1=不可能） | 参照 |

**`api_category` の 8/9 について**: `apilist.txt`（2021）は 8=出撃(2), 9=出撃(3) とする。
KC3改 のコメントは `2/8/9/10=Sortie, 6/11=Arsenal` と、10 と 11 を追加している。
**2021 以降に増えた区分とみられるが、確認していない**（`TODO(要検証)`）。

### `api_c_list[]` の要素

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_no` | number | 任務 ID | 参照 |
| `api_state` | number | `api_list` と同じとみられる | 参照 |
| `api_progress_flag` | number | 同上 | 参照 |
| `api_c_flag` | number | 省略可。**意味不明** | `TODO(要検証)` |

**`api_c_list` は 2021 年の `apilist.txt` には存在しない。** 後から追加されたフィールドである。

型定義には kcsapi.ts と ElectronicObserverEN の両方にあるが、
**本調査で確認した 3 実装のいずれも `api_c_list` を読んでいない。**
唯一の手がかりは KC3改 `Quest.js` のコメントで、

> quest F90, F98, F99, F100 also affected by `api_c_flag` in `api_c_list` array,
> which outside of `api_list`, and it may get non-zero progress even slotitem scrapping not enough

とあり、**特定の工廠系任務の進捗に関係するらしい**が、そのコメント自身が推測の形をとっている。
`TODO(要検証)`: `api_c_list` / `api_c_flag` の意味。

## 関連するエンドポイント

任務に関する `/kcsapi/` のエンドポイントは、確認した限り次の 4 つだけである
（kcsapi.ts と ElectronicObserverEN の型定義ディレクトリが一致、2026-08-02 参照）。

| パス | 内容 | 進捗の数値を含むか |
| --- | --- | --- |
| `api_get_member/questlist` | 任務一覧 | **含まない**（段階のみ） |
| `api_req_quest/start` | 任務の受注 | 出典は「情報なし」（`apilist.txt`） |
| `api_req_quest/stop` | 任務の中止 | 同上 |
| `api_req_quest/clearitemget` | 任務達成・報酬受け取り | 報酬のみ（`api_material` / `api_bounus`。**綴りは `bounus`**） |

`api_port/port` の `api_basic.api_parallel_quest_count`（最大同時受領可能任務数）は
→ [api_port_port.md](api_port_port.md)。**個々の任務の進捗は `api_port/port` には含まれない。**

## 既存ツールがどう数えているか（参照）

**3 実装とも「自前で数える」。** ゲームから届く `api_progress_flag` は補正にしか使っていない。

### 七四式電子観測儀（ElectronicObserver）

- 参照先: ElectronicObserverEN/ElectronicObserver（**MIT**。
  帰属表示 `Original work copyright (c) 2014 Andante` / `Translation work copyright (c) 2015 Ryuu Kitsune`）
  - `ElectronicObserver/Data/Quest/ProgressData.cs`（最終コミット 2026-06-21）
  - `ElectronicObserver/Data/Quest/QuestProgressManager.cs`（最終コミット 2026-06-26）

**「達成回数/最大値」の表示はここから来ている。** `ProgressData` が
`Progress`（現在値）と `ProgressMax`（最大値）を持ち、
`ToString()` が `$"{Progress}/{ProgressMax}"` を返す。
`ProgressMax` は**コード内に任務 ID ごとにハードコードされている**
（`QuestProgressManager.cs` 内に `new Progress*(...)` が約 506 箇所）。

- `Increment()` / `Decrement()` で自前に加減算する。
  `api_state != 2`（遂行中でない）なら加算しない
- 任務一覧をまだ読んでいない時点の加算は `TemporaryProgress` に退避し、後で反映する
- **永続化している**: `Settings\QuestProgress.xml`（`DataContract` でシリアライズ）

加算の契機として購読している API（`QuestProgressManager.Initialize()`）:

| API | 何を数えるか |
| --- | --- |
| `api_get_member/questlist`（応答） | 任務一覧の更新（＝進捗の補正） |
| `api_req_map/start`（応答） | 出撃開始 |
| `api_req_map/next`（応答） | 進撃 |
| `api_req_sortie/battleresult`（応答） | 戦闘結果 |
| `api_req_combined_battle/battleresult`（応答） | 連合艦隊の戦闘結果 |
| `api_req_practice/battle_result`（応答） | 演習結果 |
| `api_req_mission/result`（応答） | 遠征帰還 |
| `api_req_nyukyo/start`（**要求**） | 入渠 |
| `api_req_hokyu/charge`（応答） | 補給 |
| `api_req_kousyou/createitem`（応答） | 装備開発 |
| `api_req_kousyou/createship`（**要求**） | 建造 |
| `api_req_kousyou/destroyship`（**要求**） | 解体 |
| `api_req_kousyou/destroyitem2` | 装備廃棄（コード上の注記: 応答前に装備データが消えるため直接呼ばれる） |
| `api_req_kousyou/remodel_slot`（応答） | 改修 |
| `api_req_kaisou/powerup`（応答） | 近代化改修 |
| `api_port/port`（応答） | 進捗の保存タイマー（加算ではない） |

`api_progress_flag` の使い方は `ProgressData.CheckProgress()` のみで、
**自前カウンタを「その段階の下限まで引き上げる」補正である**
（`Math.Max(Progress, ceil(ProgressMax * 0.5))` 等）。

### KC3改（KC3Kai）

- 参照先: KC3Kai/KC3Kai（**MIT**）
  - `src/library/objects/Quest.js`（最終コミット 2025-11-25）
  - `src/library/managers/QuestManager.js`（最終コミット 2026-05-30）
  - `src/data/quests_meta.json`（最終コミット 2026-06-27）

**自前のカウンタを持つ。** `KC3Quest.tracking` が `[[現在値, 最大値], ...]` の配列で、
**最大値は `src/data/quests_meta.json` に定義されている**
（776 件の任務メタのうち **135 件**に `tracking` がある。例: 任務 214 = `[[0,36],[0,24],[0,12],[0,6]]`）。

- `increment()` を `src/library/modules/Kcsapi.js` の各 API ハンドラから呼ぶ
  （建造・開発・補給・入渠・解体・改修・近代化改修・遠征・演習・出撃結果など）
- **`KC3Quest.progress` は `api_progress_flag` をそのまま入れたもの**であり、回数ではない
  （`this.progress = data.api_progress_flag`）
- **永続化している**: `localStorage`（`KC3QuestManager.save()` / `load()`）
- **`api_progress_flag` は `autoAdjustCounter()` での補正にのみ使う。**
  他 2 実装と違い、**不足だけでなく超過も補正する**
  （自前カウンタから逆算した段階と実際の段階を比べ、食い違えば書き換える）

### KancolleSniffer

- 参照先: bitbucket.org/kancollesniffer/kancollesniffer（**Apache-2.0**。
  `Copyright (C) 2013-2021 Kazuhiro Fujieda` / `Copyright (C) 2021 hATrayflood`。
  **MIT ではない。コードの流用は避ける**（[sources.md](../sources.md) C-6））
  - `KancolleSniffer/Model/QuestInfo.cs`（最終コミット 2025-12-23）
  - `KancolleSniffer/Model/QuestCounter.cs`（最終コミット 2026-05-26）
  - `KancolleSniffer/Model/QuestCountList.cs`（最終コミット 2026-06-27、
    直近コミットは「6/26メンテでの追加任務の実装」）

**自前のカウンタを持つ。** `QuestCount.Now` / `Spec.Max` を持ち、
`ToString()` が `$"{Now}/{Spec.Max}"` を返す。
`QuestCountList` に任務 ID ごとの仕様（`Max` / 周期 / 勝利ランク条件など）が
**約 186 件ハードコード**されている。

`api_progress_flag` の扱いは `QuestInfo.InspectQuestList()` で

```
private static readonly int[] _progress = [0, 50, 80];
...
Progress = _progress[(int)entry.api_progress_flag],
```

と、**0 / 50 / 80（%）に直接読み替えている**（`api_state == 3` のときだけ 100 にする）。
`QuestCount.AdjustCount(progress)` が、その段階から取りうる下限 `low` と上限 `high` を求め、
自前カウンタがその範囲外なら範囲内に押し込む。

> **注意: KancolleSniffer の任務テーブルは独立な出典ではない。**
> `QuestCountList.cs` のコメントに
> **「このテーブルは七四式電子観測儀を参考に作成した」**と明記され、
> ElectronicObserver の `QuestProgressManager.cs` の URL が示されている（2026-08-02 確認）。
> **両者を「2 つの独立実装の一致」として扱ってはならない。**
> 独立しているのは KC3改 である。

### 攻略 Wiki の記述

[艦これ攻略 Wiki の「任務」ページ](https://wikiwiki.jp/kancolle/%E4%BB%BB%E5%8B%99)には、
多数の任務に

```
※任務進捗変化 50%(3/5)→80%(4/5)→達成(5/5)
```

という形の注記が付いている（2026-08-02 確認）。
**プレイヤー側も「段階の表示」から回数を逆算して記録している**ということであり、
ゲームが回数を出していないことの傍証になる。

同ページのコメント欄にも「50% のまま」「8 つ目の廃棄で 80%」といった、
段階でしか観測できていない報告がある（2026-06-06 の投稿、2026-08-02 確認）。

## 進捗まわりの既知のクセ（参照。すべて 2021 年時点の資料）

出典は andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/kcmemo.md`（MIT）。
**このファイルの最終コミットは 2021-04-01 で凍結している**（[sources.md](../sources.md) C-1）。
同リポジトリの `apilist.txt` は冒頭で
**「推測・噂で書いてある点も多々ある、信じすぎないこと」**と自ら断っている。
以下はその但し書きを引き継ぐ。

- **カウンタがサーバ側で共有される任務がある。**
  デイリー開発・建造任務は共用で、2 回目の操作で 50% になる。
  東京急行の遠征任務も共用で「7 回」とあるが 6 回で達成する。
  輸送船 5 隻の変則デイリーと輸送船 3 隻のデイリーは干渉し、1 隻撃沈で 2 増える
- **内部進捗が 0 から始まらない任務がある。**
  「輸送船団護衛を強化せよ！」は 1/5 から始まる（3 回で 80% になるため）
- **クライアント側で達成判定される任務がある**（`kcmemo.md` が 20 件を列挙）。
  いずれも「〜を廃棄、〜を保有」型で、**達成していても `api_state` は 2（遂行中）のまま**。
  廃棄が済んでいると `api_progress_flag` が 1 や 2 になることがあり、
  ゲーム画面側にはそれと異なる進捗率が出ることがある
- 428「近海に侵入する敵潜を制圧せよ！」は 1 エリアで 2 回成功するごとに段階が進む。
  「対潜警戒任務 2 回」なら 50%、「対潜警戒任務 1 回＋海峡警備行動 1 回」なら 0%
- 854『戦果拡張任務！「Z作戦」前段作戦』は 4 つ中 3 つで「80% 以上」になる

攻略 Wiki 側にも同種の記述がある（2026-08-02 確認）。

- B10「敵空母を撃沈せよ！」は、**受注せずに Bd4 を受注して敵空母を撃沈すると
  「進捗度 80% 以上」と表示されるが、その後に受注しても達成にならない**

> **これらはいずれも「段階から回数を逆算する」ことを難しくする要因である。**
> ゲームが回数を返さない以上、逆算には任務ごとの個別知識が要る。

## 食い違い max3 の任務の逆算結果

**同じ `api_progress_flag = 1` を、実装ごとに違う回数に逆算している。**
具体例として任務 **337（「十八駆」演習！／演習 S 勝利 3 回）**を挙げる。

| 実装 | `api_progress_flag = 1` のときの自前カウンタ | 根拠 |
| --- | --- | --- |
| **七四式電子観測儀** | **2 / 3** | `ProgressData.CheckProgress()` の `IsOldQuest` に 337 が含まれ、`ceil(3 × 0.5) = 2` に補正される |
| **KC3改** | **1 / 3** | `autoAdjustCounter()` の `maxCount === 3` 特例リストに 337 が含まれ、`api_progress_flag` の値そのもの（=1）に補正される |
| **KancolleSniffer** | **1 / 3** | 337 の `Shift = 2`。`ceil((3+2) × 0.5) − 2 = 1` |

**どれが正しいかは本調査では判断できない。**
`api_progress_flag` からは「50% 以上 80% 未満」しか分からず、
実際の内部カウンタが n/3 なのか n/5 なのかを外から見分ける手段がないためである。

- 七四式電子観測儀は `IsOldQuest` を「**50% がまだ 2/3 を意味する任務**」と定義し、
  337 / 607 / 608 / 211 / 218 など 19 件を列挙している
- KC3改 は同じ現象を「**これらの任務は 1/3 が 50% と扱われる**」と逆向きに説明し、
  337 / 339 / 350 / 356 / 357 / 368 / 607 / 608 / 674 の 9 件を列挙している
  （うち 607 / 608 は「2/3 でも 80% にならない」として補正対象から外している）
- KancolleSniffer は個別リストではなく「**`Shift` は Max=3 なら 2、Max=4 なら 1**」
  という一般則をコメントに書き、内部カウンタが n/5 で回っていると仮定している

**3 者は同じ観測（段階表示）に対して異なるモデルを置いている。**
[sources.md の「観測できない内部値」](../sources.md#1-観測できない内部値--最大の食い違い要因)と同じ構図である。

`TODO(要検証)`: 実測。任務 337 を 1 回 S 勝利した直後に任務画面を開き、
`api_progress_flag` が 1 になるかを確認すれば決着する。

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| `api_progress_flag` の意味（0 / 50% 以上 / 80% 以上）、`api_state`、`api_tab_id`、`api_list` の各フィールド、`api_req_quest/*` | andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/apilist.txt`（MIT、**最終コミット 2021-09-19**） | 2026-08-02 |
| 進捗のクセ（共有カウンタ・1/5 開始・クライアント側判定・428 / 854） | andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/kcmemo.md`（MIT、**最終コミット 2021-04-01**） | 2026-08-02 |
| レスポンスの型（`api_c_list` を含む）、リクエストパラメータ | KagamiChan/kcsapi.ts `api_get_member/questlist/{request,response}.ts`（MIT, `Copyright (c) 2018- Poi contributors.`、最終コミット 2026-05-08） | 2026-08-02 |
| 同上（C# 側の裏取り）、`api_list` 要素が `int` になりうること | ElectronicObserverEN/ElectronicObserver `ElectronicObserver.KancolleApi.Types/ApiGetMember/Questlist/`、`Models/APICList.cs`、`Models/APIListClass.cs`（MIT、最終コミット 2023-07-01） | 2026-08-02 |
| 七四式電子観測儀の自前カウンタ・購読 API・永続化先・`IsOldQuest` | ElectronicObserverEN/ElectronicObserver `ElectronicObserver/Data/Quest/ProgressData.cs`（2026-06-21）、`QuestProgressManager.cs`（2026-06-26）（MIT） | 2026-08-02 |
| KC3改 の `tracking` / `autoAdjustCounter` / `quests_meta.json` / タブ仕様のコメント | KC3Kai/KC3Kai `src/library/objects/Quest.js`（2025-11-25）、`src/library/managers/QuestManager.js`（2026-05-30）、`src/library/modules/Kcsapi.js`、`src/data/quests_meta.json`（2026-06-27）（MIT） | 2026-08-02 |
| KancolleSniffer の `_progress = [0,50,80]` / `AdjustCount` / `QuestCountList` / **EO 由来である旨のコメント** | bitbucket.org/kancollesniffer/kancollesniffer `KancolleSniffer/Model/QuestInfo.cs`（2025-12-23）、`QuestCounter.cs`（2026-05-26）、`QuestCountList.cs`（2026-06-27）（**Apache-2.0**） | 2026-08-02 |
| 「※任務進捗変化 50%(3/5)→80%(4/5)→達成(5/5)」形式の注記、B10 の挙動、コメント欄の報告 | [艦これ攻略 Wiki「任務」](https://wikiwiki.jp/kancolle/%E4%BB%BB%E5%8B%99)（ライセンス表記なし。**参照のみ、転載不可**） | 2026-08-02 |

## 未確認のこと

- `TODO(要検証)`: **実測が一件も無い。** 本プロジェクトはこのエンドポイントを観測していない。
  任務画面を開いたときの発生・サイズ・実際のフィールドは未確認
- `TODO(要検証)`: `api_c_list` / `api_c_flag` の意味。型定義には両資料にあるが、
  参照した 3 実装のいずれも読んでいない
- `TODO(要検証)`: `api_exec_type` の意味（`apilist.txt` も「？」としている）
- `TODO(要検証)`: `api_category` の 10 / 11（KC3改 のコメントにのみ現れる）
- `TODO(要検証)`: max=3 の任務で 1 回目が 50% か 2 回目が 50% か
  （→ [食い違い](#食い違い-max3-の任務の逆算結果)）
- `TODO(要検証)`: リクエストのメソッド
- `TODO(要検証)`: `api_progress_flag` に 3 以上の値が存在しないこと。
  KC3改 は `console.assert([0,1,2].indexOf(actualPFlag) !== -1)` と 0〜2 を前提にしているが、
  「それ以外は無い」ことを示す資料は見ていない
- `TODO(要検証)`: `api_req_quest/start` と `api_req_quest/stop` のレスポンス
  （`apilist.txt` は「情報なし」とだけ書いている）
