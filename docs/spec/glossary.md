# 用語集

> ステータス: **承認待ち**（2026-08-08 起草）。
> 「命名の原則」と「ゲーム用語 ↔ 英語識別子」の対応表は**人間の承認を受けていない**。
> `docs/spec/` は承認が要る階層である（[CLAUDE.md](../../CLAUDE.md) §2）。

表記ゆれを防ぐための辞書。**新しい用語を使い始めたらここに追加する**（[docs/README.md](../README.md) §6）。
エージェントは仕様書・コードの識別子でここの表記に従う。

**この文書はクラス名・型名・変数名の正である。**
実装で艦これのゲーム概念に名前を付けるときは、まずここを引く。
ここに無い語を使い始めたときは、ここに追加してから使う。

## 目次

- [プロジェクト用語](#プロジェクト用語)
- [命名の原則](#命名の原則)
- [語の衝突と裁定](#語の衝突と裁定)
- [ゲーム用語](#ゲーム用語)
- [表記の統一](#表記の統一)
- [未解決事項](#未解決事項)

## プロジェクト用語

| 用語 | 意味 |
| --- | --- |
| Harubridge | 本プロジェクトのコードネーム。正式名称は未定 |
| 専ブラ | 専用ブラウザ。特定のブラウザゲームのために作られたクライアント |
| メインウィンドウ | ゲーム画面の表示専用のウィンドウ。閉じるとアプリが終了する（[external/game-screen.md](external/game-screen.md)） |
| 情報ウィンドウ | 機能グループ単位の情報表示用ウィンドウ。メインウィンドウとは別の OS ウィンドウとして開閉・配置できる（[external/game-screen.md](external/game-screen.md)） |
| 観測 | ゲームとサーバ間の通信を読み取ること。改変は含まない（[C-02](constraints.md)） |
| 昇格 | `docs/notes/` のメモを `docs/spec/` または `docs/adr/` に反映すること |
| 縮退動作 | 未知のデータに遭遇した際、機能を一部諦めつつ停止せずに動作を続けること |

## 命名の原則

### 原則 1: 英語を既定とし、意味が壊れる語だけローマ字にする

識別子は英語で書く（[CLAUDE.md](../../CLAUDE.md) §0）。
ただし**無理に英訳すると意味が失われる語がある。** 次の順で判断する。

```
1. 英語圏の艦これ OSS（poi / KC3改）に定着した語があるか
      ある → それを使う。出典を用語表に書く
      ↓ ない
2. 一般英語の語で意味が保てるか
      保てる → 英訳する。造語であることを用語表に明記する
      ↓ 保てない
3. 英語にすると別概念と混ざる／語の輪郭が失われるか
      混ざる → ローマ字にする（例: `taiha`）。理由を用語表に書く
      ↓
4. 固有名詞（艦名・装備名・海域名）
      → 識別子にしない。表示データとして扱う（[C-05](constraints.md)）
```

**「一般にこう訳される」を根拠なく採らない。**
定訳が確認できなかったものは**こちらの造語**であり、用語表の「出典」列に `造語` と書く。
造語であること自体は問題ない。**出典があるかのように見せることが問題である。**

### 原則 2: 艦これ API の語を識別子に持ち込まない

艦これの API のフィールド名（`api_mission`、`api_deck_port`、`api_slotitem` など）は
**パース層の中だけで使う。** パース層から外に出た時点で本用語集の語に言い換える。

| 層 | 使う語 |
| --- | --- |
| 注入スクリプト / 受信・パース（[architecture.md](architecture.md) の「パース層」） | 艦これの API の語をそのまま（`api_mission`） |
| 状態・永続化・IPC・UI | **本用語集の語だけ**（`Expedition`） |

理由は 2 つある。

- **API の語はプレイヤーの語と食い違う**（→ [語の衝突と裁定](#語の衝突と裁定)）。
  そのまま持ち込むと、型名から意味が読み取れなくなる
- 非公開仕様への依存をパース層に閉じ込める方針
  （[architecture.md](architecture.md) / [C-03](constraints.md)）と一致する。
  API 名が識別子として全層に散ると、仕様変更の影響範囲が局所化できない

### 原則 3: 表記（Rust / TypeScript）

| 対象 | Rust | TypeScript |
| --- | --- | --- |
| 型・構造体・列挙・React コンポーネント | `PascalCase`（`Expedition`） | `PascalCase`（`Expedition`） |
| フィールド・変数・関数 | `snake_case`（`fleet_id`） | `camelCase`（`fleetId`） |
| 定数 | `SCREAMING_SNAKE_CASE` | `SCREAMING_SNAKE_CASE` |
| ファイル名 | `snake_case.rs` | コンポーネントは `PascalCase.tsx`、他は `camelCase.ts` |

**同じ概念は、両言語で同じ語幹にする。** `fleet_id` ↔ `fleetId` のように、
表記規則の差だけが違う状態に保つ。IPC の境界で語を変えない。

### 原則 4: 略語は 1 つの単語として扱う

`HP` `ID` `API` `URL` `LoS` などは、**大文字を連ねない。**

| 使う | 使わない |
| --- | --- |
| `Hp` / `hp` / `hpNow` | `HP` / `hP` / `HPNow` |
| `Id` / `id` / `shipId` | `ID` / `shipID` |
| `ApiClient` / `api_client` | `APIClient` |

理由: 大文字の連なりは語の境界を消し（`HPMax`）、
`camelCase` と `PascalCase` の相互変換を機械的に行えなくする。

**ただし表示文言はこの限りではない。** UI に出す文字列は `HP` と書く。
これは識別子の規則である。

### 原則 5: 単数・複数

- **型名は単数**（`Ship`、`Expedition`）。「複数の艦を表す型」を作らない
- **集合を持つ変数は複数形**（`ships`、`fleets`）
- **辞書は `<値>_by_<鍵>`**（`ship_by_id` / `shipById`）。`ship_map` としない
- **不可算名詞を集合名に使わない。** `equipments` は英語として成立しないため、
  装備は可算語の `Gear` を採る（→ [ゲーム用語](#ゲーム用語)）

### 原則 6: 同じ概念に 2 つの名前を作らない

`Ship` と `KanMusu`、`Expedition` と `Mission` を併存させない。
別名が必要になったときは、**用語表を直してから**コードを直す。

## 語の衝突と裁定

**艦これの API の語とプレイヤーの語は、複数の箇所で食い違う。**
裁定しないと、型名が何を指すか分からなくなる。

| # | 衝突 | API の語 | プレイヤーの語 | **裁定** |
| --- | --- | --- | --- | --- |
| 1 | **`mission`** | `api_mission` / `api_mst_mission` / `api_req_mission` = **遠征** | 「任務」= 達成条件つきのクエスト（API では `quest`） | **`Mission` という型・変数を作らない。** 遠征 = `Expedition`、任務 = `Quest`（→ [両論併記](#衝突-1-mission-の両論併記)） |
| 2 | **`deck`** | `api_deck_port` / `api_deck` = 艦隊 | 艦隊 / 編成 | **`Fleet`。** `deck` はパース層のみ。ただし「編成プリセット」は API が `preset_deck` と呼ぶ |
| 3 | **`remodel`** | `api_req_kaisou/remodeling` = **改造**、`api_req_kousyou/remodel_slot` = 装備の**改修（★）** | 「改修」「改造」「近代化改修」は別物 | 改造 = `Remodel` / 改修（★）= `Improvement` / 近代化改修 = `Modernization`。**3 語を混ぜない** |
| 4 | **`material`** | `api_material` は燃料・弾薬・鋼材・ボーキ**＋**高速建造材・高速修復材・開発資材・改修資材の 8 種 | 「資源」は普通この 4 種を指す | **`Resource` = 前半 4 種**、**`Consumable` = 後半 4 種**、8 種全体は `Material`（EOEN `MaterialData.cs` に倣う）。分け方は [api_port_port.md](../kancolle/api/api_port_port.md) の `api_id` 割り当てに一致する |
| 5 | **`ship`** | `api_mst_ship` は**深海棲艦も含む** | 「艦娘」は自軍のみ | `Ship` = 艦娘（自軍）。敵は `Abyssal` を前置し `AbyssalShip` とする |
| 6 | **`dock`** | `api_ndock`（入渠）と `api_kdock`（建造）が同じ「ドック」 | 入渠 / 建造 | `RepairDock` / `ConstructionDock`。**`Dock` 単体を使わない** |
| 7 | **`cond`** | `api_cond` = 疲労の内部値 | 「疲労」「キラキラ」 | 数値は `Morale`（可算・0〜100）。区分は `MoraleState`。「キラキラ」は `Sparkle` |
| 8 | **`slotitem`** | `api_slotitem` / `api_mst_slotitem` = 装備 | 装備 | `Gear`（原則 5 の不可算問題を避ける）。`slot` は**搭載枠**の意味だけに使う |
| 9 | **`type`** | `api_stype`（艦種）と `api_type`（装備の分類の配列）が別物 | 艦種 / 装備種別 | `ShipType` / `GearType`。**`Type` 単体を使わない** |
| 10 | **`kousyou`（工廠）** | `api_req_kousyou/` に建造・開発・解体・廃棄・改修が同居する | それぞれ別の操作 | `Arsenal` を使わず、`Construction` / `Development` / `Dismantle`（艦の解体）/ `Scrap`（装備の廃棄）/ `Improvement`（★）と個別に名付ける。**EOEN は建造ドックを `ArsenalData` と呼ぶが、これに倣わない** |
| 11 | **`range`** | `api_leng` = 射程、基地航空隊の行動半径は別フィールド | 射程 / 行動半径 | 射程 = `Range`（KC3 `ShipLength` → 射程）、行動半径 = `Radius`。**poi は行動半径を `Range` と訳しており、そのまま採ると衝突する** |

#### 衝突 1 `mission` の両論併記

**OSS どうしでも割れている。**

| 実装 | 遠征の呼び名 | 任務の呼び名 |
| --- | --- | --- |
| poi（MIT） | `Expedition`（`i18n/main` が `Expedition` → 遠征） | `Quest` |
| KC3改（MIT） | `Expedition`（`terms.json` の `Expedition` / `QuestFilterExped`） | `Quest` |
| **ElectronicObserverEN**（MIT） | **`MissionData.cs` / `MissionClearCondition.cs`** | `QuestData.cs` |

EOEN は**艦これの API 名に寄せて `Mission`** を採っている。
これは API のフィールド名と 1 対 1 で対応するという利点があるが、
**日本語の「任務」を英訳した語と衝突する。**

**`Expedition` を採る。** 理由は次の 2 つ。

- 本プロジェクトの文書はすべて日本語で書かれる（[CLAUDE.md](../../CLAUDE.md) §0）。
  日本語話者が `Mission` を読んだとき「任務」と解釈する確率が高く、誤読の代償が大きい
- API 名との対応は原則 2（API の語をパース層に閉じる）で別途担保されており、
  ドメイン側で API 名に寄せる利点が無い

> **`TODO(要検証)`**: 上表の API 側の語のうち、本プロジェクトで**実測できている**のは
> [docs/kancolle/api/](../kancolle/api/) に記録のある `api_port/port`・`api_start2/getData`・
> `api_get_member/questlist` の範囲に限られる。
> それ以外（`api_req_kousyou` 配下など）は**参照**であり、エンドポイント名の存在は
> [ElectronicObserverEN の型定義ディレクトリ](../kancolle/sources.md#c-2-electronicobserveren現行版)で
> 確認したが、内部のフィールド名までは確認していない。

## ゲーム用語

**「英語識別子」列がクラス名・型名・変数名の正である。**
表記は原則 3 に従って各言語へ写す（`Expedition` → Rust `expedition` / TS `expedition`）。

**「出典」列の読み方**

| 表記 | 意味 |
| --- | --- |
| `poi` | [poooi/poi](../kancolle/sources.md#c-5-poooipoi)（MIT）の英語ロケールに同じ語がある |
| `KC3` | [KC3Kai/KC3Kai](../kancolle/sources.md#c-3-kc3kaikc3kai)（MIT）の英語 UI / 識別子に同じ語がある |
| `EOEN` | [ElectronicObserverEN](../kancolle/sources.md#c-2-electronicobserveren現行版)（MIT）の型名 |
| `一般` | 艦これに固有でない一般英語。訳に判断の余地がない |
| **`造語`** | **定訳を確認できなかった。本プロジェクトの造語である** |

> 参照した OSS はいずれも MIT。ElectronicObserverEN を参照する場合は
> `Original work copyright (c) 2014 Andante` / `Translation work copyright (c) 2015 Ryuu Kitsune` の
> 帰属表示を引き継ぐ（[C-07](constraints.md) / [sources.md](../kancolle/sources.md#c-2-electronicobserveren現行版)）。
>
> **対訳データを複製したものではない。** 実装で必要になった語だけを引いた対応表である
> （[C-05](constraints.md)）。網羅は目標にしない。

**実際に参照したファイル**（いずれも 2026-08-08 に取得）

| 略記 | ファイル | ライセンス |
| --- | --- | --- |
| `poi` | `poooi/poi` の `i18n/main/ja-JP.json` / `i18n/others/ja-JP.json`。**キーが英語、値が日本語**であり、日英対応がそのまま入っている | MIT |
| `KC3` | `KC3Kai/kc3-translations` の `data/en/terms.json` と `data/jp/terms.json`。同じキーで英日を引き当てた（989 組） | MIT |
| `KC3` | `KC3Kai/KC3Kai` の `src/library/objects/Gear.js`（クラス名の根拠。[sources.md C-3](../kancolle/sources.md#c-3-kc3kaikc3kai) に既出） | MIT |
| `EOEN` | `ElectronicObserverEN/ElectronicObserver` の `ElectronicObserver/Data/` 配下の**ファイル名**（`ShipData.cs` など） | MIT |

### 提督・母港

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 提督 | `Admiral` | poi（`Admiral` → 提督）/ EOEN `AdmiralData.cs` | ゲーム文脈で `User` を使わない |
| 司令部レベル | `HqLevel` | KC3（`HQ Level:` → 司令部補正） | 原則 4 により `HQLevel` としない |
| 母港 | `HomePort` | KC3（`Home Port Notification` → 母港通知） | |
| 保有上限 | `ShipCapacity` / `GearCapacity` | **造語** | KC3 は `Max Ship Girls Available` という文で表す。単語が無い |

### 艦娘

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 艦娘 | `Ship` | poi（`Ships` → 艦娘） | **KC3 は `Ship Girls`。** 下の裁定を参照 |
| 深海棲艦・敵艦 | `AbyssalShip` | KC3（`src/data/abyssal_stats.json`） | `Ship` に敵を含めない（[裁定 5](#語の衝突と裁定)） |
| 艦種 | `ShipType` | EOEN `ShipType.cs` / KC3（`English Ship Type Codes` → 英語の艦種略号） | |
| 旗艦 | `Flagship` | poi / KC3（`Flagship Lv` → 旗艦Lv） | |
| レベル | `Level` | KC3（`Level` → レベル） | |
| 経験値 | `Exp` | KC3（`Base EXP` → 基本経験値） | |
| 耐久 | `Hp`（`hp_now` / `hp_max`） | poi（`HP` → 耐久） | 表示文言は `HP`。識別子は `Hp`（原則 4） |
| 火力 | `Firepower` | poi | |
| 雷装 | `Torpedo` | poi | |
| 対空 | `AntiAir` | poi | |
| 装甲 | `Armor` | poi | |
| 対潜 | `Asw` | poi（`ASW` → 対潜） | |
| 索敵 | `Los` | poi（`LOS` → 索敵）/ KC3（`LoS` → 索敵） | |
| 運 | `Luck` | poi | |
| 回避 | `Evasion` | **造語** | poi / KC3 の対訳では確認できなかった。`TODO(要検証)` |
| 速力 | `Speed` | KC3（`Speed` → 速力） | |
| 射程 | `Range` | KC3（`Range` → 射程） | 行動半径は `Radius`（[裁定 11](#語の衝突と裁定)） |
| ケッコン | `Married` | KC3（`Married ships cost 15% less` → ケッコン15%減少） | |
| 轟沈 | `Sunk` | KC3（`Sunk` → 轟沈） | |

> **裁定: 艦娘 = `Ship`。** KC3 は英語 UI で `Ship Girls` を使い（`Ship Girls in Service` → 艦娘保有数）、
> poi は同じ対象を `Ships` と呼ぶ（`Ships` → 艦娘）。**両者は割れている。**
> `Ship` を採るのは、`Ship` と `ShipGirl` の 2 語を併存させると原則 6 に反するためである。
> 「艦娘か、艦一般か」の区別は `Ship` / `AbyssalShip` の対で表す。

### 損傷

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 損傷区分 | `DamageState` | **造語** | |
| 無傷・かすり傷 | `Normal` | **造語** | |
| 小破 | `Shouha` | **造語**（ローマ字） | KC3 の英語 UI に用例が**見つからなかった**。中破・大破との整合で作った |
| 中破 | `Chuuha` | KC3（`Chuuha HP: {0}` → 中破の耐久値） | |
| 大破 | `Taiha` | KC3（`Annoying Taiha Alert` → 大破警告 / `Taiha HP`） | |
| 進撃 | `Advance` | KC3（`Advance Button Blocker` → 進撃ボタンブロッカー） | FR-030 |

> **裁定: 損傷区分はローマ字にする（原則 1 の 3）。**
> poi は散文で `is heavily damaged!` と書き（→ 大破した艦娘がいます）、
> KC3 は UI ラベルで `Taiha` / `Chuuha` をそのまま使う。**両者は割れている。**
>
> ローマ字を採る理由は、**大破・中破が「程度の形容」ではなく HP 比の閾値だから**である。
> 閾値には進撃警告・遠征可否といった規則がぶら下がっており、
> `Heavy` / `Moderate` と訳すと「どのくらい損傷しているか」という連続量に読める。
> **英語圏の実装が `Taiha` を訳さずに使っていること自体が、定訳が無いことの証拠である。**

### 疲労

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 疲労度（数値） | `Morale` | KC3（`Morale` → 士気 / `Morale Minimum` → 疲労度最小値）/ poi（`Morale` → 疲労回復） | **値が高いほど良い。`Fatigue` を使わない**（向きが逆になる） |
| 疲労の区分 | `MoraleState` | **造語** | |
| キラキラ | `Sparkle` | KC3（`Sparkle Notification` → ｷﾗｷﾗ発生通知 / `unsparkled ship(s)` → キラキラじゃない艦娘） | **英訳で意味が保てた例。** 動詞化もできる |
| 重疲労 | `Fatigued` | poi（`Fatigued` → 疲労） | `MoraleState` の一区分 |

### 艦隊・編成

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 艦隊・編成 | `Fleet` | poi（`Fleet` → 艦隊）/ EOEN `FleetData.cs` | API の `deck` を使わない（[裁定 2](#語の衝突と裁定)） |
| 連合艦隊 | `CombinedFleet` | KC3（`Combined Fleet` → 連合艦隊） | |
| 編成プリセット | `FleetPreset` | EOEN `FleetPresetData.cs` | |
| 装備プリセット | `GearPreset` | **造語** | `FleetPreset` との対で作った |

### 装備

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 装備 | `Gear` | KC3 `src/library/objects/Gear.js` | 下の裁定を参照 |
| 装備種別 | `GearType` | **造語**（KC3 `Gear` + EOEN `EquipmentType.cs`） | `api_type` を直接使わない（[裁定 9](#語の衝突と裁定)） |
| スロット（搭載枠） | `Slot` | KC3（`Slot` → 装備スロット） | |
| 増設スロット | `ExSlot` | KC3（`Ex-slot` → 増設スロット） | |
| 搭載数 | `SlotSize` | **造語** | |
| 改修（★） | `Improvement` / 値は `stars` | KC3（`The improvement was a success!` → 改修に成功しました / `Average Stars` → 平均改修レベル） | 改造・近代化改修と混ぜない（[裁定 3](#語の衝突と裁定)） |
| 熟練度 | `Proficiency` | KC3（`No Proficiency` → 熟練度無視） | |
| ロック | `Locked` | KC3（`Unimproved & unlocked items` → 非ロック無改修装備） | |
| 出撃札・識別札 | `EventTag` | KC3（`Ships without event tag` → 識別札が付与されてない艦娘） | FR-033 |

> **裁定: 装備 = `Gear`。3 つの実装がすべて違う語を使っている。**
>
> | 実装 | 語 |
> | --- | --- |
> | KC3改 | `Gear`（`src/library/objects/Gear.js`、クラス `KC3Gear`） |
> | ElectronicObserverEN | `EquipmentData.cs` / `EquipmentType.cs` |
> | poi | `Equip`（`Equip` → 装備） |
>
> `Gear` を採る理由は**可算名詞だから**である（原則 5）。
> `Equipment` は不可算で、集合を表す `equipments` が英語として成立しない。
> `Equip` は動詞であり、名詞として使うと「装備する」と読めてしまう。

### 資源

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 資材（8 種全体） | `Material` | EOEN `MaterialData.cs` | 日本語の「資源」の訳語にしない（[裁定 4](#語の衝突と裁定)） |
| 資源（前半 4 種） | `Resource` | [api_port_port.md](../kancolle/api/api_port_port.md)（出典は KC3 `src/library/modules/Kcsapi.js`） | |
| 燃料 | `Fuel` | KC3（`Fuel` → 燃料） | |
| 弾薬 | `Ammo` | KC3（`Ammo` → 弾薬） | |
| 鋼材 | `Steel` | KC3（`Steel: {1}` → 鋼材） | |
| ボーキサイト | `Bauxite` | KC3（`Bauxite: {2}` → ボーキ） | |
| 消耗品（後半 4 種） | `Consumable` | **造語** | KC3 は 4 つを個別に呼ぶだけで、まとめる語を持たない |
| 高速建造材 | `Torch` | [api_port_port.md](../kancolle/api/api_port_port.md)（出典は KC3 `Kcsapi.js`） | |
| 高速修復材 | `Bucket` | 同上 | |
| 開発資材 | `Devmat` | 同上 | |
| 改修資材 | `Screw` | KC3（`Screws` → 改修資材） | |
| 補給 | `Resupply` | poi（`Resupply Needed` → 補給不足）/ KC3（`Resupply` → 補給消費） | |

### 時間管理（FR-020〜FR-027）

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 遠征 | `Expedition` | poi（`Expedition` → 遠征）/ KC3（`Expedition` → 遠征） | **EOEN は `MissionData`。**[裁定 1](#衝突-1-mission-の両論併記) |
| 入渠 | `Repair` | poi（`Docking` → 入渠 / `repair completed` → 修復完了） | |
| 入渠ドック | `RepairDock` | KC3（`Repair Dock Count` → 入渠ドック数） | |
| 建造 | `Construction` | poi（`Construction` → 建造）/ KC3（`Construction Complete!` → 建造が完了しました） | |
| 建造ドック | `ConstructionDock` | **造語** | EOEN は `ArsenalData`。採らない（[裁定 10](#語の衝突と裁定)） |
| 開発 | `Development` | poi（`DevelopSuccess` → 開発成功）/ EOEN `DevelopmentData.cs` | |
| 完了予定時刻 | `completes_at` | **造語** | ゲームは残り時間ではなく絶対時刻を返す（[timers.md](../kancolle/rules/timers.md)）。`remaining` を保存しない |
| 泊地修理 | `AnchorageRepair` | **造語** | KC3 は `Akashi Repairs`（艦名由来）。固有名詞を識別子にしない（原則 1 の 4） |

### 任務（FR-050, FR-051）

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 任務 | `Quest` | poi（`Quest` → 任務(クエスト)）/ EOEN `QuestData.cs` | |
| 達成回数 | `progress_count` | **造語** | ゲームは回数を返さない。自前で数える（[quests.md](../kancolle/rules/quests.md)） |

### 出撃・戦闘（FR-030〜FR-039）

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 出撃 | `Sortie` | poi / KC3（`Sortie` → 出撃） | |
| 演習 | `Exercise` | KC3（`Exercise` → 演習 / `Exercise Fleet [1st Group]` → 演習艦隊【第一群】） | KC3 は `PvP` とも呼ぶ。`PvP` は採らない |
| 海域 | `MapArea` | EOEN `MapAreaData.cs` | KC3 は `World`（`World 1` → 鎮守府海域）。**割れている**ため、EOEN 側を採る |
| マス | `Node` | poi（`Node` → マス）/ KC3（`Map Node Markers` → マスヒント） | |
| 羅針盤 | `Compass` | EOEN `CompassData.cs` / KC3（`Compass and Battle Data` → 羅針盤・戦闘データ） | FR-038 |
| 海域ゲージ | `Gauge` | KC3（`No gauge` → ゲージなし） | FR-034 |
| 制空値 | `FighterPower` | poi（`Fighter Power` → 制空）/ KC3（`FighterPow` → 制空） | [fighter-power.md](../kancolle/formulas/fighter-power.md) |
| 基地航空隊 | `LandBase`（略 `Lbas`） | KC3（`LBAS` → 基地航空隊 / `Wider Land Base Plane View` → 基地航空隊）/ EOEN `BaseAirCorpsData.cs` | 略記は原則 4 により `Lbas` |
| 配置転換 | `Relocation` | poi（`Relocating` → 配置転換中）/ EOEN `RelocationData.cs` | |
| 行動半径 | `Radius` | **造語** | poi は `Range` → 行動半径だが、射程と衝突する（[裁定 11](#語の衝突と裁定)） |
| 戦果 | `Senka` | EOEN `SenkaLeaderboardRefreshKind.cs` | KC3 は `Rank Pts`。**ローマ字を採る**（順位点・提督経験値と紛れるため）。FR-045 |
| ドロップ | `Drop` | KC3（`Sortie Drop Faces` → 出撃ドロップ） | FR-041 |

### その他

| 日本語 | 英語識別子 | 出典 | 備考 |
| --- | --- | --- | --- |
| 図鑑 | `Library` | KC3（`Ship Library in No. Order` → 艦船図鑑No順 / `Detect New Ships via Library` → 艦娘図鑑） | FR-016 |
| 図鑑番号 | `LibraryNo` | **造語** | 対応する API フィールドを特定していない。`TODO(要検証)` |
| 中破絵 | `DamagedArt` | **造語** | FR-017。KC3 の `Damaged Ship Icons`（→ 中破以上顔グラ）は別物（一覧のアイコン） |
| ダメコン | `Damecon` | KC3（`Ships w/Damecon` → ダメコン装備艦） | **英語圏でもローマ字が定着している例** |
| 改造 | `Remodel` | poi（`RemodelLv` → 次の改造）/ KC3（`This ship can be remodeled.` → すでに改造可能） | [裁定 3](#語の衝突と裁定) |
| 近代化改修 | `Modernization` | poi（`Modernization succeeded` → 近代化改修に成功しました） | 同上 |
| 解体（艦） | `Dismantle` | **造語** | KC3 は散文で `scrapped`（→ 解体）と書くが、装備の廃棄と同じ語になる |
| 廃棄（装備） | `Scrap` | KC3（`fed, scrapped or sunk` → 解体・改修消費・轟沈） | 艦の解体と区別する |

## 表記の統一

| 使う | 使わない |
| --- | --- |
| 艦娘 | 艦、キャラ |
| 艦種 | 艦のタイプ、船種 |
| 遠征 | 派遣 |
| 入渠 | 修理、ドック入り |
| 観測 | 傍受、フック、キャプチャ |
| 提督 | ユーザー（ゲーム文脈の場合） |

> **「艦」単体は使わない**が、**複合語は既存の語として許容する**
> （艦種 / 艦隊 / 艦船図鑑 / 旗艦 など）。置き換えると通じなくなるため。

### 使ってはいけない英語識別子

**パース層の外で**次の語を使わない（原則 2 / [語の衝突と裁定](#語の衝突と裁定)）。

| 使わない | 使う | 理由 |
| --- | --- | --- |
| `Mission` | `Expedition` | 「任務」と衝突する |
| `Deck` | `Fleet` | API の語 |
| `SlotItem` | `Gear` | API の語 |
| `Equipment` / `Equip` | `Gear` | 不可算 / 動詞 |
| `Dock` 単体 | `RepairDock` / `ConstructionDock` | 入渠と建造が混ざる |
| `Type` 単体 | `ShipType` / `GearType` | 艦種と装備種別が混ざる |
| `Cond` | `Morale` | API の語 |
| `Fatigue` | `Morale` | 値の向きが逆になる |
| `ShipGirl` | `Ship` | `Ship` と二重になる |
| `Ship`（敵に対して） | `AbyssalShip` | 自軍と混ざる |
| `Arsenal` | `Construction` / `Development` ほか | 5 つの操作が混ざる |
| `HP` / `ID` / `API`（識別子中） | `Hp` / `Id` / `Api` | 原則 4 |

## 未解決事項

- `TODO(未確定)`: UI 上の表示文言をゲーム内表記に寄せるか、一般語に寄せるか。
  **本文書は識別子の正であって、表示文言の正ではない**（原則 4 の但し書き）。
  表示文言を決めたら、その所在を [`external/`](external/) 側に書く。
- `TODO(未確定)`: 「命名の原則」と「ゲーム用語」の表は**人間の承認を受けていない**（冒頭のステータス）。
- `TODO(要検証)`: **回避（`Evasion`）の定訳。** poi / KC3改の対訳データで確認できなかった。
  他のステータス（火力・雷装・対空・装甲・対潜・運）は poi に用例があるが、回避だけ無い。
- `TODO(要検証)`: **小破（`Shouha`）。** KC3改の英語 UI には `Taiha` / `Chuuha` はあるが
  `Shouha` は見つからなかった。中破・大破との整合で作った造語である。
- `TODO(要検証)`: **図鑑番号に対応する API のフィールド。** `api_sortno` との対応を確認していない。
- `TODO(要検証)`: **[裁定 3](#語の衝突と裁定)（`remodel`）と [裁定 10](#語の衝突と裁定)（`kousyou`）の
  API 側の裏付け。** `api_req_kousyou/remodel_slot` などのエンドポイント名は
  [EOEN の型定義ディレクトリ名](../kancolle/sources.md#c-2-electronicobserveren現行版)で確認したが、
  **本プロジェクトでは実測していない。**
- **網羅していない領域**（必要になってから足す。[research-kancolle](../../.claude/skills/research-kancolle/SKILL.md) の
  「網羅を目標にしない」に従う）: 戦闘の詳細（陣形・交戦形態・砲撃/雷撃/夜戦の各段階）、
  装備の細目（電探・主砲などの分類名）、家具、ランキング、`data/kancolle/` のデータ項目名。
