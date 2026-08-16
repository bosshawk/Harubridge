# kcsapi 全体の概要

> **この文書は [`_template.md`](_template.md) の節構成に従っていない。**
> テンプレートは 1 エンドポイントを記述するための型であり、
> 全体像を述べる本文書には合わないため、概要として読みやすい構成を採った。
> 出典と観測日を必ず書く規律（[../README.md](../README.md)）はそのまま適用する。

- 最終観測: 2026-08-02
- 確度: 節ごとに明示する（実測 / 参照 / `TODO(要検証)`）

## この文書の位置づけ

艦これのサーバ API（通称 kcsapi）が全体としてどういう形をしているかを記録する。
**個別のエンドポイントは別ファイルに書く**（[README.md](README.md) の一覧を参照）。

艦これの API 仕様は公式に公開されていない（[C-03](../../spec/constraints.md)）。
ここに書かれていることは**すべて観測と参照に基づく推定**であり、予告なく変わる。

## 観測方法（実測 2026-08-02）

macOS の WKWebView に XHR フックを注入し、実際にプレイした 1 セッションの通信を捕捉した。
以下「実測」と書いたものはこの観測を指す。

**捕捉したレスポンス本文はリポジトリに置いていない。**
提督名・認証トークンを含むため（[C-04](../../spec/constraints.md) /
[C-07](../../spec/constraints.md)）。本文書に載せるのは構造とサイズだけである。

## ホストと画面構成

| 事実 | 確度 |
| --- | --- |
| ゲームの入口は `https://play.games.dmm.com/game/kancolle` | 実測 2026-08-16（HTTP 200 を確認） |
| 入口ページは `x-frame-options: SAMEORIGIN` を返す | 実測 2026-08-16（HTTP ヘッダ） |
| 入口ページは別オリジンのページの `iframe` 内に描画されない。同じページ・同じ WebView に並べた `example.com` は描画される | 実測 2026-08-16（macOS / WKWebView） |
| ゲーム本体は `osapi.dmm.com` 配下の iframe で動作する | 実測 2026-08-02 |
| iframe 内のゲームサーバは `w00g` / `w13b.kancolle-server.com` | 実測 2026-08-02 |
| 2025-10-17 に稼働全 20 サーバ群が HTTPS(SSL) へ移行した | 参照（公式告知） |
| API パスは `/kcsapi/` で始まる | 実測 2026-08-02 |

`osapi.dmm.com/gadgets/` に艦これのガジェットが載る構成と、
サーバのホスト名が `*.kancolle-server.com` である点は、
KC3Kai の拡張マニフェストが対象 URL として同じパターンを列挙していることとも一致する。

`TODO(要検証)`: サーバ番号とホスト名（`w00g` / `w13b` など）の対応表。
本プロジェクトでは自分が接続した 1 サーバしか観測していない。

### 階層を切り出して表示できるか（実測 2026-08-16）

自前のローカルページに各階層を `iframe` で埋め込み、macOS / WKWebView で確かめた。

| 埋め込む対象 | 描画 | 操作 | 課金導線 |
| --- | --- | --- | --- |
| 入口ページ `play.games.dmm.com/game/kancolle` | されない（`X-Frame-Options`） | — | — |
| ガジェット枠 `osapi.dmm.com/gadgets/ifr` | **されない**（画面は白いまま。音は出る） | — | 到達できない |
| ゲーム本体 `*.kancolle-server.com/kcs2/index.php` | される | できる（母港まで到達） | **購入確認が出ない** |

- ガジェット枠は音が出るのに描画されない。**親のコンテナと協調して初めて配置される**
  作りだと考えられる（`TODO(要検証)`: 協調の具体的な仕組みは確認していない）
- 課金の購入確認は**別ウィンドウのポップアップではなく、ページ内のオーバーレイ**である
  （入口ページを丸ごと表示した状態では正常に表示されることを実測）。
  これを描くのは切り出した内側のフレームより外側であり、
  **階層を削ると描画か課金導線のどちらかが必ず壊れる**

### ゲーム画面の要素とサイズ

| 事実 | 確度 |
| --- | --- |
| 入口ページ内でゲームを載せる要素の id は `game_frame` | 参照（poi / ElectronicObserver。2026-08-16） |
| ゲーム画面の大きさは 1200 × 720 | 参照（ElectronicObserver `Browser/BrowserViewModel.cs` の `--game-frame-width` / `--game-frame-height`。2026-08-16）。1200 × 720 の枠に見切れも余白も無く収まることは実測（2026-08-16） |
| **`game_frame` はゲーム画面より背が高い。** ゲームの下に配信元のリンク帯（作戦要綱・艦娘・用語・情報・運営電文）を含む | 実測 2026-08-16 |

`game_frame` の実寸を測って縮尺の基準にすると、リンク帯まで含めて縮むため
ゲームが小さくなり、リンク帯が画面に出る。**縮尺の基準はゲーム画面の大きさ
（1200 × 720）に取り、余りは切り落とす。**

既存ツールはいずれも**階層を切り出さず**、入口ページを丸ごと読み込んだうえで
`<style>` を注入し、配信元ページの装飾を隠して `game_frame` を左上に固定している。

| ツール | 該当箇所 | ライセンス |
| --- | --- | --- |
| poi | `assets/js/page-align.js` | MIT |
| ElectronicObserver | `Browser/BrowserViewModel.cs` | MIT |
| logbook-kai | ブラウザを内蔵せず、プロキシとして観測するのみ | MIT |

## 通信の形

| 事実 | 確度 |
| --- | --- |
| 捕捉した API 通信は**全件 XHR**。`fetch` と WebSocket は 1 件も検出されなかった | 実測 2026-08-02 |
| レスポンス本文はすべて文字列 `svdata=` で始まる | 実測 2026-08-02 |
| `svdata=` を除去すると JSON としてパースできる | 実測 2026-08-02 |

`svdata=` の 7 文字を落としてから JSON にかける処理は、
poi（`lib/game-api-broadcaster.ts`）と KC3Kai（`src/library/modules/Network.js`）の
双方が同じ形で実装しており、本プロジェクトの実測と一致する。

`TODO(要検証)`: リクエスト側の形式とパラメータ。今回の観測ではレスポンスのみを記録し、
リクエストボディを解析していない。認証パラメータの有無・名前も未確認
（値は記録しない。[C-04](../../spec/constraints.md)）。

## 封筒（トップレベル）構造

**観測した全レスポンスで、トップレベルのキー数は 3 だった**（実測 2026-08-02）。

参照した OSS では、その 3 つは次の名前である（参照）。

| キー | 型 | 意味 | 確度 |
| --- | --- | --- | --- |
| `api_result` | number | 処理結果コード | 参照 |
| `api_result_msg` | string | 結果メッセージ | 参照 |
| `api_data` | object / array / null | 本体。エンドポイントごとに構造が異なる | 参照 |

- `api_result` / `api_result_msg`: KC3Kai `Network.js` が `svdata=` 除去後の JSON から
  この 2 つを読んでいる。kcsapi.ts の `api_get_member/payitem` にも
  `{ api_data: null, api_result: number, api_result_msg: string }` という
  失敗時の型が定義されている。
- `api_data`: KC3Kai `Kcsapi.js` が全エンドポイントで `response.api_data` を参照している。

`api_result` の値について:

- `TODO(要検証)`: 正常時の値。参照した資料に「正常時は N」という明示的な記述を見つけられなかった。
- 異常時に `100` や `201` といった値をとることがある（参照: ElectronicObserver `kcmemo.md`）。
  同資料は「JSON ですらない HTML が返ってくることもある」とも記している。

**したがって、パースは失敗しうる前提で書く必要がある**（[C-03](../../spec/constraints.md) の縮退動作）。

## エンドポイントの区分

パスの第 1 セグメントで役割が分かれている（参照 + 実測）。

| 接頭辞 | 役割 | 確度 |
| --- | --- | --- |
| `api_start2/` | マスタデータ。プレイヤーに依存しない静的な定義 | 参照 + 実測 |
| `api_get_member/` | プレイヤー保有データの取得（艦・装備・ドック・図鑑など） | 参照 + 実測 |
| `api_req_*/` | プレイヤーの操作に対する処理と結果（`api_req_kaisou/` `api_req_member/` など） | 参照 |
| `api_port/` | 母港。[`api_port_port.md`](api_port_port.md) を参照 | 参照 + 実測 |

### `api_get_master/` について

**実測 2026-08-02 では `api_get_master/` 配下のエンドポイントを 1 件も観測しなかった。**
また、参照した 3 つの OSS（kcsapi.ts / ElectronicObserver / KC3Kai）のいずれにも
`api_get_master/` という文字列は存在しなかった（2026-08-02 時点の各 master ブランチ）。

現在マスタデータは [`api_start2/getData`](api_start2_getdata.md) に集約されているとみられる。

`TODO(要検証)`: 過去に `api_get_master/` が存在し、`api_start2/getData` に統合されたのかどうか。
統合の経緯を示す一次資料を確認できていない。

## 観測したエンドポイント（実測 2026-08-02）

**1 セッションで観測されたものだけを載せる。** 網羅ではない
（[../README.md](../README.md) の「網羅しようとしない」）。
サイズはレスポンス本文のバイト数（`svdata=` を含む）。

| パス | サイズ (bytes) | 発生回数 | 内容 |
| --- | ---: | ---: | --- |
| `/kcsapi/api_start2/getData` | 2,332,462 | 1 | マスタデータ。→ [api_start2_getdata.md](api_start2_getdata.md) |
| `/kcsapi/api_port/port` | 271,824 | 7 | 母港。→ [api_port_port.md](api_port_port.md) |
| `/kcsapi/api_get_member/require_info` | 188,731 | 1 | 起動時情報群（装備・工廠ドック・アイテム・家具など） |
| `/kcsapi/api_get_member/picture_book` | 56,707 | 1 | 図鑑 |
| `/kcsapi/api_get_member/ndock` | 764 | 1 | 入渠ドック |
| `/kcsapi/api_get_member/preset_deck` | 753 | 1 | 編成記録（プリセット） |
| `/kcsapi/api_start2/get_option_setting` | 175 | 1 | 母港スキンと音量設定 |
| `/kcsapi/api_get_member/preset_dev_items` | 84 | 1 | 装備プリセット関連 |
| `/kcsapi/api_req_member/get_incentive` | 82 | 1 | 褒賞 |
| `/kcsapi/api_req_kaisou/can_preset_slot_select` | 81 | 1 | 装備プリセット展開可否 |
| `/kcsapi/api_get_member/payitem` | 71 | 2 | 保有課金アイテム |

観測できたことの整理:

- **サイズの偏りが極端である。** `api_start2/getData` と `api_port/port` の 2 つで
  総バイト数のほとんどを占める（実測 2026-08-02）。
- `api_port/port` は**1 セッション中 7 回発生し、毎回同程度のサイズだった**（実測 2026-08-02）。
  差分ではなく全量が飛んでいるとみられる。
- 71 〜 84 bytes の応答は、封筒 3 キーと数個のフィールドだけの小さな JSON である。
  kcsapi.ts の型でも `api_req_kaisou/can_preset_slot_select` は `{ api_flag: number }`、
  `api_get_member/preset_dev_items` は `{ api_max_num: number }` の 1 フィールドのみ（参照）。

### 小さいエンドポイントの中身（参照）

| パス | `api_data` の形 | 出典 |
| --- | --- | --- |
| `api_start2/get_option_setting` | `api_skin_id`, `api_volume_setting`（`api_bgm` / `api_se` / `api_voice` / `api_duty` / `api_be_left`） | kcsapi.ts |
| `api_get_member/preset_dev_items` | `api_max_num` | kcsapi.ts |
| `api_req_kaisou/can_preset_slot_select` | `api_flag` | kcsapi.ts |
| `api_req_member/get_incentive` | `api_count`, `api_item[]`（`api_mode` / `api_type` / `api_mst_id` / `api_getmes` / `api_slotitem_level`） | ElectronicObserver apilist.txt |
| `api_get_member/payitem` | `api_payitem_id`, `api_name`, `api_description`, `api_type`, `api_count`, `api_price`。**未保有時は `api_data` が `null`** | kcsapi.ts + apilist.txt |

## API が呼ばれる順番（参照）

ElectronicObserver `apilist.txt` は、ログイン時の呼び出し順を次のように記している。

```
api_req_member/get_incentive
api_start2/getData
api_get_member/require_info
api_port/port
```

本プロジェクトの実測でもこの 4 つはすべて観測された（実測 2026-08-02）。
ただし**観測した順序が上記と厳密に一致するかは確認していない**（`TODO(要検証)`）。
また実測では `api_start2/get_option_setting` も観測されており、
これは `apilist.txt` の順序表には現れない。

`api_port/port` は帰投時（出撃・遠征）にも呼ばれる（参照: apilist.txt）。
実測で 7 回発生したことと整合する。

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| 観測したエンドポイント・サイズ・回数、`svdata=`、トップレベル 3 キー、XHR のみ、iframe のホスト | 実測（WKWebView への XHR フック注入） | 2026-08-02 |
| 2025-10-17 の全 20 サーバ HTTPS(SSL) 移行 | 公式 X 投稿 [@KanColle_STAFF](https://x.com/KanColle_STAFF/status/1979119622491377969) | 2026-08-02 |
| 封筒キー `api_result` / `api_result_msg` | KC3Kai/KC3Kai `src/library/modules/Network.js`（MIT） | 2026-08-02 |
| 封筒キー `api_data` | KC3Kai/KC3Kai `src/library/modules/Kcsapi.js`（MIT） | 2026-08-02 |
| 封筒 3 キーの型（失敗時） | KagamiChan/kcsapi.ts `api_get_member/payitem/response.ts`（MIT, Copyright (c) 2018- Poi contributors.） | 2026-08-02 |
| `svdata=` の 7 文字除去 | poooi/poi `lib/game-api-broadcaster.ts`（MIT） | 2026-08-02 |
| `api_result` の異常値（100 / 201）、HTML が返る場合 | andanteyk/ElectronicObserver `ElectronicObserver/Other/Information/kcmemo.md`（MIT） | 2026-08-02 |
| 小エンドポイントの `api_data` 構造 | KagamiChan/kcsapi.ts 各 `response.ts`（MIT） | 2026-08-02 |
| ログイン時の呼び出し順、`get_incentive` / `payitem` の中身 | andanteyk/ElectronicObserver `apilist.txt`（MIT） | 2026-08-02 |
| `osapi.dmm.com/gadgets/` と `*.kancolle-server.com` の URL パターン | KC3Kai/KC3Kai `src/manifest.json`（MIT） | 2026-08-02 |

> `kcsapi.ts` の README によれば、これは Phase 2 のサンプルレスポンスから
> 型を推論して生成したものである。**推論元のサンプルがいつのものかは不明**であり、
> 現在のゲームと差がある可能性がある（`TODO(要検証)`）。
>
> `ElectronicObserver` の `apilist.txt` は冒頭で
> 「2014/08 夏イベント〜の API の情報」「推測・噂で書いてある点も多々ある、信じすぎないこと」
> と自ら断っている。**本文書で「参照」とした記述はこの但し書きを引き継ぐ。**

## 未確認のこと

- `TODO(要検証)`: リクエストの形式・パラメータ名（値は記録しない）
- `TODO(要検証)`: `api_result` の正常時の値
- `TODO(要検証)`: サーバ番号とホスト名の対応
- `TODO(要検証)`: ログイン時の呼び出し順序の実測確認
- `TODO(要検証)`: `api_get_master/` の存否と歴史的経緯
- `TODO(未観測)`: 出撃・戦闘・建造・遠征系のエンドポイント。
  今回のセッションでは母港での操作しか行っておらず、観測できていない
