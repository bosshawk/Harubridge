# ADR-0023: 当面は 1 アカウントのみを扱い、切り替わりの検出だけを実装する

- ステータス: **Proposed**
- 日付: 2026-08-03
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0021](0021-data-persistence.md)（未解決事項「複数アカウントの扱い（別 ADR で決める）」）,
  [ADR-0022](0022-observed-data-privacy.md)（識別子を保存物に載せない）,
  [ADR-0005](0005-publish-as-oss.md),
  [docs/spec/constraints.md](../spec/constraints.md)（C-04 / C-07）,
  [docs/spec/requirements.md](../spec/requirements.md)（未解決事項「複数アカウント（複数プレイヤー）を扱うか」/ NFR-008 / NFR-011）,
  [docs/kancolle/api/api_port_port.md](../kancolle/api/api_port_port.md)

## 背景と課題

**1 台の端末で複数の提督アカウントを切り替えて使えるようにするか。**
これは [ADR-0021](0021-data-persistence.md) と
[requirements.md](../spec/requirements.md) の双方に未解決事項として残っており、
**データの持ち方の根に効く**ため、実装に入る前に決める必要がある。

論点は 2 段ある。

1. そもそも複数アカウントを扱うか
2. 扱う／扱わないにかかわらず、**アカウントの同一性を何で判定するか**

2 が難しい。艦これ側が提督を指す値は `api_member_id`（提督 ID）だが、
本プロジェクトは C-07（リポジトリは公開前提。提督名・ID を一度でもコミットしない）と
C-04（取得データを外部に出さない）の下にある。
そして [ADR-0021](0021-data-persistence.md) 系の検討では
**「提督名・会員 ID・トークンは保存しない」**という整理が既に置かれている
（`docs/notes/persistence-comparison.md`。git 管理外のため本文書に結論だけ引く）。
つまり **`api_member_id` を素材にして保存先の名前を作ることは、既存の方針と正面から衝突する。**

### 前提として効いている事実

- `api_member_id` は `api_basic` だけでなく、`api_material` の各要素・`api_deck_port` の各要素にも
  含まれる（[api_port_port.md](../kancolle/api/api_port_port.md)。参照, 2026-08-02）。
  **1 箇所で落とせば済む値ではない。**
- [ADR-0021](0021-data-persistence.md) は、**任務の達成回数を `events/*.jsonl` から自前で数える**と決めている。
  これは「事象の積み上げが唯一の正」であることを意味する。
- ゲーム画面は DMM のページを WebView に表示する構成であり
  （[architecture.md](../spec/architecture.md) / [external/game-screen.md](../spec/external/game-screen.md)）、
  ログイン状態は配信元のセッションに属する。アプリは資格情報を持たない。
- [requirements.md](../spec/requirements.md) は承認済みだが、
  **複数アカウントに対応する FR は 1 件も無い。**

### 既存の専ブラがどうしているか（実コードを読んで確認した）

いずれも 2026-08-03 に各リポジトリを取得し、該当ファイルを読んで確認した。
コミットは下表のものを参照している。

| 実装 | アカウントの扱い | 確認した根拠 |
| --- | --- | --- |
| **KC3改** | **記録だけを提督 ID で分ける。現在の状態は 1 組しか持たない** | 下記 |
| **poi** | **切り替わりを検出して状態を捨てる。任務進捗はファイル名に提督 ID を埋める** | 下記 |
| **KancolleSniffer** | **区別できない**（提督 ID をパース前に削除している） | 下記 |
| **ElectronicObserver** | **区別しない**（保存先は固定。提督 ID を他所から参照していない） | 下記 |

**KC3改**（`KC3Kai/KC3Kai` master `b32fe46`, 2026-07-22, MIT）

- IndexedDB の全テーブルが `hq` 列を持ち、書き込みも読み出しも
  `where("hq").equals(this.index)` で絞っている（`src/library/modules/Database.js`）。
  `account` テーブルの定義は `"++id,&hq,server,mid,name"`
- その `hq` は **`api_member_id` そのもの**である。
  `src/library/objects/Player.js` が `this.id = data.mid; KC3Database.index = this.id;` としている
- **ただし現在の状態（艦隊・装備・消耗品・基地）は `localStorage` に 1 組しか持たない。**
  `PlayerManager.js` の `localStorage.fleets` / `localStorage.consumables` などに `hq` は入らず、
  `Player.js` も `localStorage.player` に「いまの提督」1 人分を書く。
  つまり**別アカウントでログインすると現在の状態は上書きされ、過去の記録だけが `hq` で残る**
- `account` テーブルは `Database.js` が書き込むだけで、**リポジトリ内のどこからも読まれていない。**
  アカウントを一覧して選ぶ画面は存在しない
- **バックアップのファイル名に提督名がそのまま入る。**
  `DataBackup.js` は `[提督名] YYYY-MM-DD.kc3data` を作る
- インポート時には、ファイル内の `player.id` が現在の `hq` と異なると確認ダイアログを出す
  （`src/pages/strategy/tabs/profile/profile.js`）

**poi**（`poooi/poi` master `a515460`, 2026-07-31, MIT）

- `api_get_member/require_info` を受けたとき、`api_basic.api_member_id` が前回と異なれば
  **`basic` 以外の state をすべて捨てる**（`views/redux/info/index.ts`）。
  切り替わりを検出して混ざらないようにしているが、**並行して保持はしない**
- 一方で任務の進捗は
  **`APPDATA_PATH/quest_tracking_<api_member_id>.cson` というファイル名で保存される**
  （`views/redux/info/quests/persistence.ts`）。
  **提督 ID がファイル名として OS 上に現れる**

**KancolleSniffer**（Bitbucket `kancollesniffer/kancollesniffer` master `e05ff02` = v16.16, 2026-08-01）

> **GitHub の `fujieda/KancolleSniffer` を根拠にしてはならない。**
> あれは v12.11（2021-06-18）で止まったミラーであり、開発は Bitbucket にある
> （[sources.md C-6](../kancolle/sources.md#c-6-kancollesnifferbitbucket-が現行)）。
> 本節は現行 master で読み直した結果である。

- `Privacy.cs` が `api_member_id` をレスポンスから**正規表現で削除**する。
  除去は本処理に渡す前に行われる。
  **したがって内部にアカウントを区別する手段が原理的に無い**
- 設定・ログの保存先は `Config.cs` の `BaseDir`（= `AppInfo.BaseDir`、実行ディレクトリ）固定
- **この 2 点は v12.11 から v16.16 まで変わっていない。** 結論に影響は無い

**ElectronicObserver**（`andanteyk/ElectronicObserver` develop `469015b`, 2021-11-23, MIT）

- `Data/AdmiralData.cs` に `AdmiralID`（`api_member_id`）プロパティはあるが、
  **リポジトリ全体を検索してもこのプロパティを参照する箇所が 1 つも無い**
- 記録の保存先は `Resource/Record/RecordManager.cs` の `MasterPath = "Record"` 固定。
  アカウントによる分岐は無い

**整理すると、4 実装のうち「記録をアカウントで分けている」のは KC3改 だけであり、
その KC3改 ですら現在の状態は 1 組しか持たない。**
そして**アカウントを分けている 2 実装（KC3改・poi）は、いずれも提督 ID か提督名を
ファイル名に露出させている。**

### 艦これ側について分かっていること・いないこと

| 記述 | 確度 |
| --- | --- |
| 1 つの提督アカウントは 1 つのサーバに所属する。運営による転属（異動願い）が行われたことがある | **参照（二次情報）**: 艦隊これくしょん攻略 Wiki\*「サーバー」（wikiwiki.jp、2026-08-03 参照） |
| サーバ番号とホスト名の対応 | 不明（[overview.md](../kancolle/api/overview.md) でも `TODO(要検証)`） |
| **サーバ転籍で `api_member_id` が変わるか** | `TODO(要検証)`。**本決定の切り替え検出の誤検出に直結する** |
| DMM / 艦これの規約が、同一人物による複数アカウント保有をどう扱うか | `TODO(要検証)`。**規約原文を確認できていない**（[constraints.md](../spec/constraints.md) の未解決事項と同じ） |

**規約について推測で書かない。** 二次情報には「DMM は複数アカウントを禁止している」という
記述があるが、艦これに適用される規約の原文を確認できていないため、
**本決定は規約を理由にしていない。**

## 決定

**当面は 1 アカウントのみを扱う。複数アカウントの同時保持・切り替えは実装しない。**
ただし**アカウントが切り替わったことは検出し、黙って混ぜない。**
そのための識別子は、**`api_member_id` そのものを保存も命名も一切しない形**で持つ。

具体的には次の 4 点を決める。

1. **複数アカウントの並行保持を実装しない。** UI にプロファイル選択を出さない。
2. **起動時と `api_port/port` 受信時に、観測したアカウントが前回と同じかを照合する。**
   異なると判定したら、**記録（`events/` への追記と `state/` の更新）を止め、
   ユーザーに知らせる。** 既存データへ書き足さない。
3. **照合に使う値は `api_member_id` のソルト付きハッシュとし、`state/` 内の
   JSON の 1 フィールドとしてのみ持つ。**
   - ソルトは**保存先の初期化時に生成した乱数**をローカルに置く。リポジトリにも配布物にも含めない
   - **ハッシュ値をディレクトリ名・ファイル名に使わない**
   - `api_member_id` の平文はどこにも書かない（[ADR-0021](0021-data-persistence.md) の許可リストを通さない）
4. **保存ルートの直下にプロファイル層を 1 段挟み、当面は `default` 固定とする。**

   ```text
   <root>/profiles/default/state/*.json
   <root>/profiles/default/events/YYYY-MM.jsonl
   <root>/profiles/default/replays/YYYY-MM/<id>.json
   ```

   `<root>` の決め方は [ADR-0021](0021-data-persistence.md) の規律 10 のまま
   （debug は `.local/`、release は `app_local_data_dir()`）。
   **パス解決を 1 関数に閉じる**という規律もそのまま維持する。

### いま何を決めておけば、後で複数対応に踏み切っても困らないか

複数対応の判断そのものは先送りするが、**先送りできない（後から入れると移行が要る）のは
次の 2 つだけ**である。本決定はこの 2 つだけを先に固定する。

| いま決めること | 後回しにすると何が起きるか |
| --- | --- |
| **パスにプロファイル層を 1 段挟む** | 公開後に層を足すと、既存ユーザーのデータを移動する移行処理が要る。**実装未着手のいま入れるコストはゼロである** |
| **アカウントの同一性を照合できる値を最初から持つ** | 持たずに始めると、混ざったあとで**分離できない**。`api_member_id` を保存していないので、遡って仕分ける材料が無い |

逆に、**いま決めなくてよいもの**を明示しておく。

- プロファイルの**表示名**をどうするか（ユーザーが付けるのか、自動で付けるのか）
- ディレクトリ名に何を使うか（`default` 以外が必要になってから決める）
- アカウント横断の集計を出すかどうか

### なぜ「混ぜない」が必須なのか

[ADR-0021](0021-data-persistence.md) により、**任務の達成回数は `events/*.jsonl` を数えて出す。**
別アカウントの事象が同じファイルに混ざると、任務カウンタは静かに誤った値を出し続ける。
そして**混ざったあとで分離する手段が無い**（識別子を保存していないため）。

**これは「不便」ではなく「不可逆な破損」である。** 検出だけは値段が安く、効果が大きい。

## 検討した選択肢

### 論点 1: 複数アカウントを扱うか

#### 案 A1: 当面 1 アカウントのみ。切り替わりは検出する（採用）

- 概要: 並行保持は実装しない。別アカウントを検出したら記録を止めて知らせる。
- 利点:
  - **要求にない機能を実装しない**（[requirements.md](../spec/requirements.md) に該当 FR が無い）
  - 不可逆な破損（任務カウンタの混入）だけは防げる
  - 識別子を保存・命名に使わずに済むため、C-07 の漏洩経路が生まれない
  - poi が同じ判断（検出して state を捨てる）を実装しており、前例がある
- 欠点:
  - **家族などで 1 台を共有する場合、後から使うほうの記録が取れない。**
    これはユーザーに見える不便であり、外部仕様に書く必要がある
  - 検出が誤爆すると（サーバ転籍で ID が変わる等）記録が止まる。
    誤爆時にユーザーが復帰できる操作が要る

#### 案 A2: 最初から複数アカウントに対応する

- 概要: プロファイルを複数持ち、UI で切り替える。
- 利点: 1 台を共有する利用に最初から応えられる。KC3改 が記録の分離を実現している。
- 却下理由: 対応する要求が [requirements.md](../spec/requirements.md) に無く、
  **同時に 2 アカウントを開けるかどうか（WebView のセッション分離が可能か）も確認していない**ため。

#### 案 A3: アカウントを区別しない（KancolleSniffer / ElectronicObserver 方式）

- 概要: 提督 ID を見ない。単一の保存先に書き続ける。
- 利点: 最も単純で、識別子を一切扱わないため C-04 / C-07 の観点で最も安全。
  実際に 2 実装がこの形で長期間運用されている。
- 却下理由: [ADR-0021](0021-data-persistence.md) が**任務の達成回数を事象の積み上げから数える**と
  決めているため、別アカウントの事象が混ざるとカウンタが**不可逆に壊れる**ため。
  KancolleSniffer は提督 ID を除去したうえで任務進捗を扱っており条件が異なる。

### 論点 2: アカウントの同一性を何で判定するか

#### 案 B1: `api_member_id` をディレクトリ名・ファイル名に使う（KC3改 / poi 方式）

- 概要: `profiles/<api_member_id>/` のように、提督 ID をパスの一部にする。
- 利点: 実装が最も単純。前例が 2 つある。
- 却下理由: **ファイル名は不具合報告のスクリーンショットやファイル一覧に必ず現れる**ため。
  poi は現に `quest_tracking_<api_member_id>.cson` を、
  KC3改 は `[提督名] YYYY-MM-DD.kc3data` を OS 上に作っている。C-07 の下では採れない。

#### 案 B2: `api_member_id` を平文で `state/` に保存する

- 概要: パス名には使わないが、照合のため値そのものを持つ。
- 却下理由: 「提督名・会員 ID・トークンは保存しない」という
  [ADR-0021](0021-data-persistence.md) 系の許可リストの整理に反し、
  [ADR-0022](0022-observed-data-privacy.md) の二次防御（既知の識別子を落とす拒否リスト）が
  自分の保存データを弾いてしまうため。

#### 案 B3: ソルト付きハッシュを `state/` の 1 フィールドとして持つ（採用）

- 概要: ローカル生成の乱数ソルトと `api_member_id` からハッシュを作り、照合にだけ使う。
- 利点:
  - 保存されるのは**照合できるだけの値**であり、元の ID を復元できない。
    ソルトがローカル固有なので、**万一コミットされても第三者は総当たりできない**
  - パス名に出ないため、スクリーンショットやファイル一覧から漏れない
  - 複数対応に踏み切るときも、この値をそのままプロファイルの内部キーに使える
- 欠点:
  - **ソルトを失うと過去のプロファイルと照合できなくなる**（別アカウントとして扱われる）
  - 保存先を丸ごと別環境にコピーしたときの挙動を決めておく必要がある

#### 案 B4: ユーザーがプロファイル名を付ける

- 概要: 「自分用」「弟用」のような名前をユーザーに付けさせ、それで分ける。
- 却下理由: **切り替わりの検出に使えない**ため。ユーザーが付け替えを忘れれば混入し、
  案 A3 と同じ不可逆な破損が起きる。
  （ただし**複数対応に踏み切るときの「表示名」としては有効**であり、未解決事項に送る）

## 決め手

**アカウントの取り違えによる不可逆なデータ破損を防ぐことと引き換えに、
複数アカウントを同時に扱うという機能そのものを当面捨てた。**

## 影響

- 実装への影響:
  - パス解決関数が返すルートに `profiles/default/` を含める
    （[ADR-0021](0021-data-persistence.md) の規律 10 の下位に置く。**分岐するのは引き続きルートだけ**）
  - パース層に、`api_member_id` をハッシュに落として照合する処理を置く。
    **平文はそこから内側へ渡さない**（[ADR-0022](0022-observed-data-privacy.md) の除去境界と同じ位置）
  - 不一致を検出したときの縮退（記録の停止 + 通知）は、
    [ADR-0021](0021-data-persistence.md) の規律 9（記録機能だけを無効化して縮退）と同じ形にする
- ドキュメントへの影響（いずれも**人間の承認が必要**）:
  - [requirements.md](../spec/requirements.md) の未解決事項
    「複数アカウント（複数プレイヤー）を扱うか」を、本 ADR への参照に置き換える
  - **別の提督でログインしたときに何が起きるかはユーザーに見える振る舞いである。**
    [docs/spec/external/](../spec/external/) に `E-nn` として定義する必要がある。
    対応する FR が無いため、要求の追加が要る可能性がある
  - [architecture.md](../spec/architecture.md)「データの持ち方」に、
    プロファイル層と識別子を持たない原則を書く
  - [ADR-0021](0021-data-persistence.md) の未解決事項
    「複数アカウントの扱い（別 ADR で決める）」は本 ADR で解消される
    （ADR 本文は書き換えない。一覧側で参照する）
- 取り消す場合のコスト:
  - **複数対応へ広げる方向は低い。** プロファイル層が既にあり、照合用の値も既にある
  - **アカウントを区別しない方向（案 A3）へ戻すのも低い。** 照合を止めるだけで済む
  - **`api_member_id` をパス名に使う方向（案 B1）へ戻すのは不可逆。**
    一度ユーザーの環境にその名前でディレクトリが作られると、後から消せない

## 未解決事項

- `TODO(要検証)`: **サーバ転籍（異動願い）で `api_member_id` が変わるか。**
  変わるなら本決定の検出が誤爆する。転籍は頻繁ではないが、
  **誤爆したときにユーザーが「同じ提督である」と宣言して復帰できる操作**が要る。
  その操作は外部仕様の対象である
- `TODO(要検証)`: 規約（DMM / 艦これ）が同一人物の複数アカウント保有をどう扱うか。
  原文を確認できていない。[constraints.md](../spec/constraints.md) の未解決事項と合わせて確認する
- `TODO(要検証)`: WebView のセッション（DMM のログイン状態）を分離できるか。
  できないなら、案 A2 は「切り替え」までしか実現できず「同時」は原理的に不可能
- `TODO(未確定)`: ソルトの置き場所と生成の契機。保存先を別環境にコピーしたときの挙動
- `TODO(未確定)`: 複数対応に踏み切るときのプロファイル表示名（案 B4 を再検討する）
- `TODO(未確定)`: 検出したときの既定動作。「記録を止める」以外に
  「新しいプロファイルを作る」提案を出すかどうか。外部仕様側の判断

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| KC3改 の `hq` 列・`account` テーブル・`hq` = `api_member_id`・`localStorage.player`・バックアップ名・インポート時の ID 確認 | KC3Kai/KC3Kai master `b32fe46`（MIT）: `src/library/modules/Database.js` / `src/library/objects/Player.js` / `src/library/managers/PlayerManager.js` / `src/library/modules/DataBackup.js` / `src/pages/strategy/tabs/profile/profile.js` | 2026-08-03 |
| poi の提督切り替え検出と `quest_tracking_<api_member_id>.cson` | poooi/poi master `a515460`（MIT）: `views/redux/info/index.ts` / `views/redux/info/quests/persistence.ts` / `lib/env.ts` | 2026-08-03 |
| KancolleSniffer が `api_member_id` を除去すること・保存先が実行ディレクトリ固定であること | **Bitbucket** `kancollesniffer/kancollesniffer` master `e05ff02`（= v16.16, 2026-08-01, Apache-2.0）: `KancolleSniffer/Privacy.cs` / `KancolleSniffer/Config.cs` | 2026-08-03 |
| ElectronicObserver の `AdmiralID` が未参照であること・`MasterPath = "Record"` 固定 | andanteyk/ElectronicObserver develop `469015b`（MIT）: `ElectronicObserver/Data/AdmiralData.cs` / `ElectronicObserver/Resource/Record/RecordManager.cs` | 2026-08-03 |
| `api_member_id` が `api_basic` / `api_material` / `api_deck_port` に現れること | [api_port_port.md](../kancolle/api/api_port_port.md)（同文書の出典を引き継ぐ） | 2026-08-02 |
| 1 アカウント = 1 サーバ、転属制度の存在 | 艦隊これくしょん攻略 Wiki\*「サーバー」（wikiwiki.jp）。**二次情報であり、公式の一次資料ではない** | 2026-08-03 |

> **ElectronicObserver は現行の開発線を読めていない。** `andanteyk/ElectronicObserver`
> （上表）は 2023-10-05 を最後に更新が止まり、開発は
> `ElectronicObserverEN/ElectronicObserver`（= `gre4bee` 系、既定ブランチ `main`、
> 最終 push 2026-07-28）に移っている。**そちらでの `AdmiralID` の扱いは未確認**であり、
> 本文書の EO に関する記述は 2021 年時点の `develop` に対するものである。
> なお当該フォークは GitHub がライセンスを判定できていない（`NOASSERTION`）ため、
> **参照する前にライセンス本文を確認すること**。`TODO(要検証)`
>
> KancolleSniffer については現行 master（v16.16）で読み直し済み。
