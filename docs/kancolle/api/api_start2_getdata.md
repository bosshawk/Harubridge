# API: api_start2/getData

- パス: `/kcsapi/api_start2/getData`
- 最終観測: 2026-08-02
- 確度: 実測（発生条件・サイズ）+ 参照（マスタの種類と構造）

**マスタデータ一式。** 艦・装備・海域・遠征などの、プレイヤーに依存しない定義が入る。
全体像は [overview.md](overview.md) を参照。

## いつ飛ぶか

ログイン時に 1 回。ElectronicObserver `apilist.txt` はログイン時の順序を
`api_req_member/get_incentive` → `api_start2/getData` → `api_get_member/require_info` →
`api_port/port` と記している（参照）。

**実測 2026-08-02: 1 セッション中 1 回のみ観測した。** 母港・編成・入渠の操作では再発生しなかった。

`TODO(要検証)`: 再ログインせずに再取得される契機があるか（メンテ明けなど）。

## リクエスト

| 項目 | 内容 |
| --- | --- |
| メソッド | `TODO(要検証)`（今回の観測ではレスポンスのみ記録した） |
| 形式 | `TODO(要検証)` |
| 主なパラメータ | `TODO(要検証)`。kcsapi.ts は `api_verno` のみを挙げる（参照） |

**`api_token` など認証情報の値は記録しない**（[C-04](../../spec/constraints.md)）。

## レスポンス

- 形式: `svdata=` + JSON（実測 2026-08-02）
- 観測したサイズ: **2,332,462 bytes（約 2.3 MB）**（実測 2026-08-02）。
  今回観測した中で**最大**である
- トップレベルは封筒 3 キー（`api_result` / `api_result_msg` / `api_data`）。
  → [overview.md](overview.md#封筒トップレベル構造)

### `api_data` に含まれるマスタ（参照）

kcsapi.ts の `APIStart2GetDataResponse` が挙げる 21 個。
意味は ElectronicObserver `apilist.txt` による。

| キー | 型 | 内容 | 確度 |
| --- | --- | --- | --- |
| `api_mst_ship` | array | **艦船データ。艦娘と深海棲艦の両方が同じ配列に入る** | 参照 |
| `api_mst_shipgraph` | array | 艦船の画像設定（ファイル名・バージョン・表示座標） | 参照 |
| `api_mst_stype` | array | 艦種（カテゴリ）。1=海防艦, 2=駆逐艦, 3=軽巡洋艦, … | 参照 |
| `api_mst_shipupgrade` | array | 特殊改装の設定（改装設計図・カタパルト等の所要数） | 参照 |
| `api_mst_slotitem` | array | 装備データ | 参照 |
| `api_mst_slotitem_equiptype` | array | 装備カテゴリ | 参照 |
| `api_mst_equip_ship` | array | 特殊装備。特定の艦に対する装備可否の上書き | 参照 |
| `api_mst_equip_exslot` | number[] | 補強スロットに装備可能なカテゴリ | 参照 |
| `api_mst_equip_exslot_ship` | object | 補強スロットへの特殊装備（`api_slotitem_id` / `api_ship_ids`） | 参照 |
| `api_mst_equip_limit_exslot` | object | — | `TODO(要検証)` |
| `api_mst_maparea` | array | 出撃海域カテゴリ（鎮守府海域・南西諸島海域など）。`api_type` 0=通常, 1=イベント | 参照 |
| `api_mst_mapinfo` | array | 海域詳細（海域名・作戦名・難易度・出撃編成フラグなど） | 参照 |
| `api_mst_mapbgm` | array | 海域 BGM の設定 | 参照 |
| `api_mst_mission` | array | 遠征データ（遠征名・詳細・獲得資源の目安・編成アイコン） | 参照 |
| `api_mst_furniture` | array | 家具データ | 参照 |
| `api_mst_furnituregraph` | array | 可動家具のプロパティ | 参照 |
| `api_mst_useitem` | array | アイテムデータ（使用形態・カテゴリ・説明） | 参照 |
| `api_mst_payitem` | array | 課金アイテム（アイテム屋さん）のデータ | 参照 |
| `api_mst_item_shop` | object | アイテム屋さんの品揃え（`api_cabinet_1` / `api_cabinet_2`。-1 は空き） | 参照 |
| `api_mst_bgm` | array | 母港 BGM（`api_id` / `api_name`） | 参照 |
| `api_mst_const` | object | 各種上限値（`api_parallel_quest_max` / `api_boko_max_ships` / `api_dpflag_quest`） | 参照 |

`TODO(要検証)`: この 21 個が 2026 年時点の実データと一致するか。
**実測ではサイズしか記録しておらず、キーの一覧を数えていない。**
kcsapi.ts は Phase 2 のサンプルから型を推論したものであり、生成時期が不明である。

### 特に注意が要る点（参照）

- **`api_mst_ship` は艦娘と深海棲艦を混在させる。** 深海棲艦の要素は
  `api_sort_id` が 0 で、`api_yomi` に敵クラス（`""` / `"-"` / `"elite"` / `"flagship"`）が入り、
  艦娘にあるステータス項目の多くを持たない。**フィールドの欠落を前提にすること**
  （[C-03](../../spec/constraints.md) の縮退動作）。
- **衣替え艦娘のデータは `api_mst_shipgraph` にしか存在しない**（参照: apilist.txt）。
- **`api_mst_stype[].api_equip_type` のキーは `"1"` のような数字始まりの名前を持つ。**
  apilist.txt はこれを「規約違反であるため正規の JSON パーサでは正常に読み込めない」と
  注意している。`api_mst_equip_ship` / `api_mst_equip_exslot_ship` /
  `api_mst_equip_limit_exslot` も同様に、kcsapi.ts では
  `{ [key: string]: ... }` の辞書として型付けされている（参照）。
  → **構造体ではなくマップとして読む必要がある。**
- `api_mst_mapinfo.api_max_maphp` / `api_required_defeat_count` は
  「進捗によって変更されるため、これらをあてにせず `mapinfo` を参照すること」と
  apilist.txt が注意している（参照）。
- `api_mst_furnituregraph` は kcsapi.ts で省略可能（`?`）になっている（参照）。
  **存在しない場合がある。**
- KC3Kai `Master.js` は「`api_mst_mapcell` は KC Phase 2 以降で削除された」と
  コメントしている（参照）。**過去の資料に出てくるキーが現在も存在するとは限らない。**

### 主要マスタの主なフィールド（参照）

網羅しない。実装で最初に要るものだけ挙げる。

#### `api_mst_ship`（艦娘）

`api_id`（艦船固有 ID）, `api_sortno`（図鑑番号）, `api_sort_id`（母港ソート順）,
`api_name`, `api_yomi`, `api_stype`（艦種 ID）, `api_ctype`（艦型 ID）,
`api_afterlv`（改装 Lv）, `api_aftershipid`（改装後 ID。**文字列**。`"0"`=なし）,
`api_taik`（耐久。`[0]`=初期値, `[1]`=最大値）, `api_souk`（装甲）, `api_houg`（火力）,
`api_raig`（雷装）, `api_tyku`（対空）, `api_luck`（運）,
`api_soku`（速力。0=陸上基地, 5=低速, 10=高速, 15=高速+, 20=最速）,
`api_leng`（射程。0=無, 1=短, 2=中, 3=長, 4=超長）,
`api_slot_num`（スロット数）, `api_maxeq`（艦載機搭載数）,
`api_buildtime`（建造時間。分単位）, `api_broken`（解体資材）, `api_powup`（近代化改修強化値）,
`api_backs`（レアリティ）, `api_fuel_max` / `api_bull_max`（搭載燃料 / 弾薬）。

`api_tais`（対潜）は**護衛空母にのみ存在する**（参照: apilist.txt）。

#### `api_mst_stype`

`api_id`, `api_sortno`, `api_name`, `api_equip_type`（装備可能カテゴリのフラグ）,
`api_scnt`（入渠時間係数）, `api_kcnt`（建造時のシルエット）。

#### `api_mst_mission`（遠征）

`api_id`, `api_disp_no`（表示上の遠征 ID。`"A1"` のような文字列）, `api_maparea_id`,
`api_name`, `api_details`, `api_reset_type`（0=通常, 1=マンスリー）,
`api_win_mat_level`（獲得資源量の目安。0〜4 を `[燃料, 弾薬, 鋼材, ボーキサイト]` で）,
`api_return_flag`（中止可否）, `api_sample_fleet`（編成アイコン `[6]`。0=空欄, 他は艦種 ID）。

#### `api_mst_const`

`api_parallel_quest_max` / `api_boko_max_ships` / `api_dpflag_quest` の 3 つ。
いずれも `{ api_int_value, api_string_value }` という同じ形をとる（参照: kcsapi.ts）。

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| サイズ 2,332,462 bytes、1 セッション 1 回、`svdata=` 形式、封筒 3 キー | 実測（WKWebView への XHR フック注入） | 2026-08-02 |
| `api_data` 直下の 21 キーと型（辞書型・省略可否を含む）、`api_mst_const` の形、リクエストパラメータ | KagamiChan/kcsapi.ts `api_start2/getData/response.ts` および `request.ts`（MIT, Copyright (c) 2018- Poi contributors.） | 2026-08-02 |
| 各マスタの意味と主なフィールド、艦娘/深海棲艦の混在、数字始まりキーの注意、ログイン時の呼び出し順 | andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/apilist.txt`（MIT） | 2026-08-02 |
| `api_mst_mapcell` が Phase 2 で削除されたこと | KC3Kai/KC3Kai `src/library/modules/Master.js`（MIT） | 2026-08-02 |

> 出典の性質と但し書きについては [overview.md](overview.md#根拠) を参照。
> `apilist.txt` は「推測・噂で書いてある点も多々ある」と自ら断っている。

## 未確認のこと

- `TODO(要検証)`: `api_mst_equip_limit_exslot` の意味。apilist.txt に記述が無い
- `TODO(要検証)`: 実データのキー一覧が上記 21 個と一致するか（実測で数えていない）
- `TODO(要検証)`: 2.3 MB の内訳。どのマスタが大部分を占めるか
- `TODO(要検証)`: 各マスタの要素数
- `TODO(要検証)`: リクエストのメソッド・形式
- `TODO(要検証)`: ログイン以外に再取得される契機の有無
- `TODO(要検証)`: `api_mst_const.api_dpflag_quest` の意味（apilist.txt も空欄）
