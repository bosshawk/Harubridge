# API: api_port/port

- パス: `/kcsapi/api_port/port`
- 最終観測: 2026-08-02
- 確度: 実測（発生条件・サイズ・回数）+ 参照（フィールド構造）

母港画面の情報一式。**本プロジェクトが依存する中心のエンドポイントである。**
全体像は [overview.md](overview.md) を参照。

## いつ飛ぶか

| 契機 | 確度 |
| --- | --- |
| ログイン直後（`api_get_member/require_info` の後） | 参照 + 実測 |
| 出撃から帰投したとき | 参照 |
| 遠征から帰投したとき（`api_req_mission/result` の後） | 参照 |

**実測 2026-08-02: 1 セッション中 7 回発生した。** サイズは毎回 271,824 bytes 前後で、
差分ではなく**そのつど全量が飛んでいる**とみられる。

`TODO(要検証)`: 7 回それぞれの契機。観測ログには発生時刻とサイズしか残しておらず、
どの操作に対応するかを紐づけていない。

## リクエスト

| 項目 | 内容 |
| --- | --- |
| メソッド | `TODO(要検証)`（今回の観測ではレスポンスのみ記録した） |
| 形式 | `TODO(要検証)` |
| 主なパラメータ | `TODO(要検証)`。kcsapi.ts は `api_port` / `api_sort_key` / `api_verno` / `spi_sort_order` を挙げる（参照）。`spi_sort_order` は綴りが不自然であり、生成元サンプル由来の誤りの可能性がある |

**`api_token` など認証情報の値は記録しない**（[C-04](../../spec/constraints.md)）。

## レスポンス

- 形式: `svdata=` + JSON（実測 2026-08-02）
- 観測したサイズ: **271,824 bytes**（実測 2026-08-02、7 回とも同程度）
- トップレベルは封筒 3 キー（`api_result` / `api_result_msg` / `api_data`）。
  → [overview.md](overview.md#封筒トップレベル構造)

以下は `api_data` 直下の構造である。

### `api_data` の主なフィールド

**すべてを埋めようとしない。** 分かっているものだけ書く。

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_basic` | object | 艦隊司令部情報（提督名・Lv・階級・保有上限など） | 参照 |
| `api_deck_port` | array | **艦隊情報。遠征の状態もここに含まれる** | 参照 |
| `api_ndock` | array | 入渠ドック | 参照 |
| `api_material` | array | 資源・消費アイテムの保有量 | 参照 |
| `api_ship` | array | **所属している全艦船の状態** | 参照 |
| `api_log` | array | 母港下の通知欄 | 参照 |
| `api_combined_flag` | number | 連合艦隊の編成種別。0=未結成, 1=機動部隊, 2=水上部隊, 3=輸送部隊。負値は強制解隊 | 参照 |
| `api_p_bgm_id` | number | 母港 BGM の ID | 参照 |
| `api_parallel_quest_count` | number | 最大同時受領可能任務数 | 参照 |
| `api_dest_ship_slot` | number | 解体時の装備廃棄フラグ。0=保管, 1=廃棄 | 参照 |
| `api_plane_info` | object | 基地航空隊関連。**要素があるときのみ存在** | 参照 |
| `api_event_object` | object | イベント海域関連。イベント期間により内容が異なる | 参照 |
| `api_c_flag` | number | — | `TODO(要検証)` |
| `api_c_flags` | number[] | — | `TODO(要検証)` |
| `api_furniture_affect_items` | object | 家具の効果に関するもの（`api_payitem_dict` を持つ） | `TODO(要検証)` |

`api_c_flag` / `api_c_flags` / `api_furniture_affect_items` は kcsapi.ts の型定義に存在するが、
**ElectronicObserver の `apilist.txt` に説明が無く、意味を確認できなかった。**
名前からの推測は書かない。

### `api_material`（資源）

要素は `{ api_member_id, api_id, api_value }`。`api_id` の割り当ては次のとおり（参照）。

| `api_id` | 内容 |
| --- | --- |
| 1 | 燃料 |
| 2 | 弾薬 |
| 3 | 鋼材 |
| 4 | ボーキサイト |
| 5 | 高速建造材（バーナー） |
| 6 | 高速修復材（バケツ） |
| 7 | 開発資材 |
| 8 | 改修資材（ネジ） |

KC3Kai `Kcsapi.js` の `api_port/port` ハンドラも、先頭 4 件を資源、
5 〜 8 件目を消耗品（torch / buckets / devmats / screws）として同じ順に読んでいる（参照）。

### `api_deck_port`（艦隊と遠征）

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_id` | number | 艦隊番号 | 参照 |
| `api_name` | string | 艦隊名 | 参照 |
| `api_ship` | number[] | 所属艦船の固有 ID。**空きは -1** | 参照 |
| `api_mission` | number[] | **遠征状況。** `[0]`=状態（0=未出撃, 1=遠征中, 2=遠征帰投, 3=強制帰投中）, `[1]`=遠征先 ID, `[2]`=帰投時刻, `[3]`=0 | 参照 |
| `api_flagship` | string | — | `TODO(要検証)` |
| `api_name_id` | string | — | `TODO(要検証)` |
| `api_member_id` | number | 提督 ID | 参照 |

要素数は保有艦隊数と等しく、パディングは無い（参照: apilist.txt の `api_get_member/deck`）。

`api_ship` の長さは基本的に 6 だが、過去のイベントで 7 になった例がある（参照）。
**固定長を前提にしない。**

### `api_ndock`（入渠ドック）

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_id` | number | ドック番号 | 参照 |
| `api_state` | number | 状態。-1=ロック（未解放）, 0=空き, 1=入渠中 | 参照 |
| `api_ship_id` | number | 入渠中の艦船の固有 ID。空きは 0 | 参照 |
| `api_complete_time` | number | 入渠完了時刻 | 参照 |
| `api_complete_time_str` | string | 入渠完了日時（文字列） | 参照 |
| `api_item1` | number | 消費燃料 | 参照 |
| `api_item2` | number | 消費弾薬（0 になる） | 参照 |
| `api_item3` | number | 消費鋼材 | 参照 |
| `api_item4` | number | 消費ボーキサイト（0 になる） | 参照 |

同じ構造が `api_get_member/ndock` としても飛ぶ（実測 2026-08-02 で 764 bytes を観測）。

### `api_ship`（所属艦船）

サイズの大部分を占めるのはこの配列とみられる（`TODO(要検証)`: 内訳は計測していない）。

| フィールド | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_id` | number | **艦船固有 ID**（この個体を指す） | 参照 |
| `api_ship_id` | number | **艦船 ID**（艦の種類。マスタ `api_mst_ship.api_id` に対応） | 参照 |
| `api_lv` | number | Lv | 参照 |
| `api_exp` | number[] | `[0]`=累積経験値, `[1]`=次の Lv まで, `[2]`=経験値バー割合 | 参照 |
| `api_nowhp` / `api_maxhp` | number | 現在 HP / 最大 HP | 参照 |
| `api_cond` | number | コンディション（疲労度） | 参照 |
| `api_fuel` / `api_bull` | number | 搭載燃料 / 搭載弾薬 | 参照 |
| `api_slot` | number[] | 装備の固有 ID。**空きは -1** | 参照 |
| `api_slot_ex` | number | 補強スロット。0=未解放, -1=未装備 | 参照 |
| `api_onslot` | number[] | 艦載機の現在搭載数 | 参照 |
| `api_slotnum` | number | スロット数 | 参照 |
| `api_ndock_time` | number | 入渠に要する時間（ミリ秒） | 参照 |
| `api_ndock_item` | number[] | 入渠時の消費資材。`[0]`=燃料, `[1]`=鋼材 | 参照 |
| `api_kyouka` | number[] | 近代化改修状態。`[0]`=火力, `[1]`=雷装, `[2]`=対空, `[3]`=装甲, `[4]`=運, `[5]`=耐久, `[6]`=対潜 | 参照 |
| `api_karyoku` / `api_raisou` / `api_taiku` / `api_soukou` | number[] | 火力 / 雷装 / 対空 / 装甲。`[0]`=現在値（装備込み）, `[1]`=最大値 | 参照 |
| `api_kaihi` / `api_taisen` / `api_sakuteki` / `api_lucky` | number[] | 回避 / 対潜 / 索敵 / 運。同上 | 参照 |
| `api_soku` | number | 速力。0=基地, 5=低速, 10=高速, 15=高速+, 20=最速 | 参照 |
| `api_leng` | number | 射程。0=無, 1=短, 2=中, 3=長, 4=超長 | 参照 |
| `api_locked` | number | 保護ロックの有無 | 参照 |
| `api_locked_equip` | number | ロックされた装備を装備しているか | 参照 |
| `api_sally_area` | number | 出撃海域。**イベント中のみ存在** | 参照 |
| `api_sortno` | number | 図鑑番号 | 参照 |
| `api_backs` | number | レアリティ | 参照 |
| `api_srate` | number | — | `TODO(要検証)`（apilist.txt も「改装☆？」と疑問符付き） |
| `api_sp_effect_items` | array | 特殊効果アイテム（`api_kind` / `api_houg` / `api_raig` / `api_souk` / `api_kaih`） | `TODO(要検証)`（kcsapi.ts に型はあるが apilist.txt に説明が無い） |

**`api_id`（個体）と `api_ship_id`（艦種）を取り違えないこと。** 名前が紛らわしい。

### `api_basic`（司令部情報）

`api_get_member/basic` と同じ構造に `api_large_dock`（大型艦建造可否）が加わる（参照）。

主なもの: `api_member_id`（提督 ID）, `api_nickname`（提督名）, `api_level`,
`api_rank`（階級。1 から 元帥 / 大将 / 中将 / 少将 / 大佐 / 中佐 / 新米中佐 / 少佐 / 中堅少佐 / 新米少佐）,
`api_experience`, `api_max_chara`（最大保有可能艦娘数）, `api_max_slotitem`（最大保有可能装備数）,
`api_count_deck` / `api_count_kdock` / `api_count_ndock`（保有艦隊数 / 工廠ドック数 / 入渠ドック数）,
`api_fcoin`（家具コイン）, `api_st_win` / `api_st_lose`（出撃勝敗）,
`api_ms_count` / `api_ms_success`（遠征回数 / 成功回数）,
`api_pt_win` / `api_pt_lose`（演習勝敗）, `api_medals`（甲種勲章保有数）（すべて参照）。

**`api_nickname` と `api_member_id` は提督個人を識別する。**
ログ・スクリーンショット・エクスポートに載せる場合はマスキングが必須
（[C-04](../../spec/constraints.md) / [C-07](../../spec/constraints.md)）。

`TODO(要検証)`: `api_nickname_id` / `api_comment_id` / `api_active_flag` /
`api_firstflag` / `api_pvp` / `api_playtime` の意味。apilist.txt も空欄になっている。

### `api_log`（通知欄）

`{ api_no, api_type, api_state, api_message }`。
`api_type` は文字列で、1 から 入渠 / 工廠 / 遠征 / 支給? / 演習 / 勲章? / 出撃 / 任務? /
申請? / 昇格? / 図鑑 / 達成? / 改造?（参照。**「?」は出典側が付けた不確かさの印**）。

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| 発生回数 7 回、サイズ 271,824 bytes、`svdata=` 形式、封筒 3 キー | 実測（WKWebView への XHR フック注入） | 2026-08-02 |
| `api_data` 直下のフィールド一覧と省略可能性 | KagamiChan/kcsapi.ts `api_port/port/response.ts`（MIT, Copyright (c) 2018- Poi contributors.） | 2026-08-02 |
| リクエストパラメータ名 | KagamiChan/kcsapi.ts `api_port/port/request.ts`（MIT） | 2026-08-02 |
| 各フィールドの意味、`api_material` の ID 割り当て、`api_deck_port.api_mission`、`api_ndock`、`api_ship`、`api_basic`、`api_log`、呼び出し契機 | andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/apilist.txt`（MIT） | 2026-08-02 |
| `api_material` の先頭 4 = 資源 / 5〜8 = 消耗品 という読み方 | KC3Kai/KC3Kai `src/library/modules/Kcsapi.js`（MIT） | 2026-08-02 |

> 出典の性質と但し書きについては [overview.md](overview.md#根拠) を参照。
> `apilist.txt` は「推測・噂で書いてある点も多々ある」と自ら断っている。

## 未確認のこと

- `TODO(要検証)`: `api_c_flag` / `api_c_flags` / `api_furniture_affect_items` の意味
- `TODO(要検証)`: `api_srate` / `api_sp_effect_items` の意味
- `TODO(要検証)`: `api_basic` の空欄フィールド群（`api_nickname_id` など）
- `TODO(要検証)`: リクエストのメソッド・形式・パラメータ、`spi_sort_order` の綴り
- `TODO(要検証)`: 7 回の発生それぞれの契機
- `TODO(要検証)`: 271,824 bytes の内訳（`api_ship` が占める割合）
- `TODO(要検証)`: 上記フィールド一覧が 2026 年時点の実データと一致するか。
  kcsapi.ts / apilist.txt はいずれも生成・更新時期が古い可能性がある。
  **本文書のフィールド表は「参照」であって「実測」ではない**
