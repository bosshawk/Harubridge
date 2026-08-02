# 艦これの情報源

- 最終確認: **2026-08-02**（全エントリをこの日に実在確認した）

**艦これについて何かを調べるときに、最初に開く文書である。**
[`research-kancolle`](../../.claude/skills/research-kancolle/SKILL.md) スキルの手順 2 から参照される。

艦これの仕様は公式に公開されていない（[C-03](../spec/constraints.md)）。
したがって**どこを見るかの選択が、そのまま記述の確度になる。**
順序を間違えると、古い資料を新しい事実として書いてしまう。

> **これはリンク集ではない。** 各情報源が「どれだけ速く」「どれだけ正確で」
> 「参照してよいか」を評価した資料である。評価には必ず根拠を書いている。
> 根拠が確認できなかったものには `TODO(要検証)` を付けた。

## 目次

- [評価の軸](#評価の軸)
- [調べる順序](#調べる順序)
- [一覧（速さ・精度・扱いやすさ）](#一覧速さ精度扱いやすさ)
- [A. 公式](#a-公式)
- [B. 有志の検証 wiki](#b-有志の検証-wiki)
- [C. OSS 実装](#c-oss-実装)
- [D. 計算ツール](#d-計算ツール)
- [E. 歴史的資料](#e-歴史的資料)
- [F. 参照してはいけないもの](#f-参照してはいけないもの)
- [G. 止まっている・消えている情報源](#g-止まっている消えている情報源)
- [食い違いが起きやすい領域](#食い違いが起きやすい領域)
- [この文書の鮮度管理](#この文書の鮮度管理)
- [確認できなかったこと](#確認できなかったこと)

## 評価の軸

### 1. 速さ — ゲームの更新がどれだけ早く反映されるか

**根拠は「実際のタイムスタンプ」で示す。**「活発そう」では評価にならない。

| 何を見たか | どう使うか |
| --- | --- |
| wiki の `RecentChanges` の日時 | 更新が止まっていないかの直接の証拠 |
| GitHub の `pushed_at` と**最新リリースの日付** | この 2 つは乖離する。リリースが止まった OSS がある |
| **ファイル単位の最終コミット日** | リポジトリが生きていても、そのファイルは死んでいることがある |
| 直近コミットのメッセージ | 現行イベント名が出ていれば、当日対応している証拠になる |

**リポジトリ全体の鮮度と、参照したいファイルの鮮度は別物である。**
本調査で最も重要な発見はこれであった（[C-1](#c-1-electronicobserver-本家andanteyk) を参照）。

### 2. 精度 — 誰がどう検証したか

| 区分 | 意味 |
| --- | --- |
| **公式** | 権利者の発表。事実だが、仕様の数値・計算式はまず出ない |
| **一次（実測）** | 自分で通信を観測した。本プロジェクトの実測がこれにあたる |
| **一次に近い（OSS 実装）** | 実装者が観測して書いたコード。**動いているという事実が検証を兼ねる** |
| **二次（検証 wiki）** | 有志が集計・検証した結果の集約。出典を辿れることが多い |
| **三次（攻略ブログ・まとめ）** | 上記を引用したもの。**計算式の出典としては使わない** |

**孫引きで確度は上がらない。** 元の資料が「推測を含む」と断っているなら、
それを引き継いで書く（[research-kancolle](../../.claude/skills/research-kancolle/SKILL.md) の規律）。

### 3. 扱いやすさ — ライセンスと機械可読性

本プロジェクトは MIT（[ADR-0011](../adr/0011-license-mit.md)）。
[C-05](../spec/constraints.md) / [C-07](../spec/constraints.md) により、
**参照した OSS のライセンスを確認し、必要な帰属表示を行う**義務がある。

ライセンスは推測せず、次のコマンドで確認した。

```sh
gh api repos/{owner}/{repo} --jq '.license.spdx_id, .archived, .pushed_at'
gh api repos/{owner}/{repo}/contents/LICENSE --jq '.content' | base64 -d
```

> **`spdx_id` が `NOASSERTION` でも「ライセンス無し」とは限らない。**
> 著作権表示行を書き足しただけで GitHub は判定を諦める。**本文を読むこと。**
> 本調査では `ElectronicObserverEN/ElectronicObserver` と `Nishisonic/logbook` が
> これに該当し、**本文はいずれも逐語の MIT** であった。
>
> 逆に、**`spdx_id` が `NONE` は本当にライセンスが無い。** この場合は
> **参照も流用も不可**である（[F 節](#f-参照してはいけないもの)）。

## 調べる順序

**上から順に当たる。** 下の段は上の段を補強するために使い、上の段を上書きしない。

```
0. docs/kancolle/ に既にあるか確認する         ← 同じことを二度調べない
      ↓
1. 自分で観測する（実測）                       ← API 構造なら最優先。確度が最も高い
      ↓
2. 機械可読な型定義で構造を裏取りする           ← kcsapi.ts / EOEN KancolleApi.Types
      ↓
3. 網羅的なフィールド一覧で名前の意味を引く      ← apilist.txt（2021 で凍結。但し書きあり）
      ↓
4. 計算式は検証 wiki で当たりを付ける            ← wikiwiki.jp/kancolle
      ↓
5. OSS 実装 2 つ以上と突き合わせる               ← KC3Kai と ElectronicObserver は独立実装
      ↓
6. 食い違ったら両論併記する                      ← 黙って片方を採らない
```

用途別の入口は次のとおり。

| 調べたいもの | 最初に見るもの | 裏取り |
| --- | --- | --- |
| **API のエンドポイントとフィールド** | 実測 → [kcsapi.ts](#c-4-kagamichankcsapits) | [apilist.txt](#c-1-electronicobserver-本家andanteyk) / [EOEN の型](#c-2-electronicobserveren現行版) |
| **計算式** | [wikiwiki.jp/kancolle](#b-1-艦隊これくしょん--艦これ--攻略-wiki) | [KC3Kai](#c-3-kc3kaikc3kai) と [ElectronicObserver](#c-2-electronicobserveren現行版) の**両方** |
| **ゲームのルール**（枠数・時間・上限） | [wikiwiki.jp/kancolle](#b-1-艦隊これくしょん--艦これ--攻略-wiki) | 実測 |
| **ゲームの更新があったか** | [@KanColle_STAFF](#a-1-kancolle_staff公式-x) | wiki の `RecentChanges` |
| **仕様変更に他ツールがどう対応したか** | [各 OSS の直近コミット](#c-oss-実装) | [艦これツール Wiki](#b-2-艦これ-ツール-wiki) |
| **通信方式・接続まわりの変化**（SSL 化など） | [艦これツール Wiki の SSL 対応表](#b-2-艦これ-ツール-wiki) | [@KanColle_STAFF](#a-1-kancolle_staff公式-x) |

**API 構造については、実測が他のどの資料よりも強い。**
参照資料はいずれも過去のどこかの時点のゲームを写したものであり、
現在のゲームと一致している保証がない。

## 一覧（速さ・精度・扱いやすさ）

評価は `◎ > ○ > △ > ×`。**根拠は各節に書いてある。** ここは索引である。

| 情報源 | 種別 | 得られるもの | 速さ | 精度 | 扱いやすさ | ライセンス |
| --- | --- | --- | :-: | :-: | :-: | --- |
| [@KanColle_STAFF](#a-1-kancolle_staff公式-x) | 公式・一次 | 更新情報 | ◎ | ◎ | × | 権利者に帰属 |
| [公式サイト（DMM）](#a-2-艦これ公式サイトdmm) | 公式・一次 | 更新情報 | △ | ◎ | × | 権利者に帰属 |
| [艦これ攻略 Wiki](#b-1-艦隊これくしょん--艦これ--攻略-wiki) | 二次・検証 | 計算式 / ルール | ◎ | ○ | △ | 明示なし。**転載不可** |
| [艦これツール Wiki](#b-2-艦これ-ツール-wiki) | 二次 | ツール対応状況 | ○ | ○ | △ | 明示なし |
| [ElectronicObserver（本家）](#c-1-electronicobserver-本家andanteyk) | 一次に近い | API 構造 / 仕様メモ | **×** | ○ | ○ | MIT |
| [ElectronicObserverEN](#c-2-electronicobserveren現行版) | 一次に近い | API 型 / 計算式 | ◎ | ○ | ◎ | MIT（本文確認） |
| [KC3Kai](#c-3-kc3kaikc3kai) | 一次に近い | 計算式 / 敵データ | ◎ | ○ | ○ | MIT |
| [kcsapi.ts](#c-4-kagamichankcsapits) | 一次に近い | API の型定義 | ○ | ○ | ◎ | MIT |
| [poi](#c-5-poooipoi) | 一次に近い | 通信の扱い方 | ○ | ○ | ○ | MIT |
| [KancolleSniffer（Bitbucket）](#c-6-kancollesnifferbitbucket-が現行) | 一次に近い | 遠征 / 任務 / 計算 | ◎ | ○ | ○ | **Apache-2.0** |
| [航海日誌拡張版](#c-7-nishisoniclogbook航海日誌拡張版) | 一次に近い | ダメージ計算 / 疲労 | △ | ○ | ○ | MIT（本文確認） |
| [aaci-prop](#c-8-nishisonicaaci-prop) | 一次に近い | 対空 CI の確率 | ○ | ○ | ◎ | MIT |
| [制空権シミュレータ v2](#d-1-制空権シミュレータ-v2) | ツール | 計算結果の突き合わせ | ○ | ○ | **×** | **無し。参照不可** |
| [艦これ API を叩く（C86）](#e-1-艦これapiを叩く) | 歴史的資料 | 過去の API 構造 | × | △ | △ | 明示なし |
| [ぜかましねっと](#g-4-三次情報攻略ブログ) | 三次 | 攻略情報 | ◎ | △ | × | 明示なし |
| [kcwikizh/kcdata](#f-1-kcwikizhkcdata) | — | — | — | — | **×** | **無し。参照不可** |

## A. 公式

**公式は「何かが変わった」ことを最も速く伝えるが、「どう変わったか」の詳細は出さない。**
数値・計算式・API 構造を公式から得ることは期待できない。

### A-1. @KanColle_STAFF（公式 X）

- URL: https://x.com/KanColle_STAFF
- 名義: 「艦これ」開発/運営
- 得られるもの: **更新情報のみ**（メンテナンス、アップデート、イベント開始・終了、
  サーバ構成の変更）。仕様の詳細は出ない
- 種別: 公式・一次

**速さ: ◎ — 最速である。根拠は公式サイト自身の記述。**
[公式サイト](#a-2-艦これ公式サイトdmm)が
「着任制限の開放・アップデート・メンテナンスなど『艦隊これくしょん』の最新情報は
公式 X(旧 Twitter) アカウント『＠KanColle_STAFF』にてお知らせしています」と明記している
（確認: 2026-08-02）。**運営自身が「ここが一次窓口である」と宣言している。**

**精度: ◎（ただし範囲が狭い）** — 権利者の発表であり、書かれていることは事実である。
一方で**仕様の詳細は原則として書かれない。**
実例として、2025-10-17 の全 20 サーバ群の HTTPS(SSL) 移行は
[api/overview.md](api/overview.md) が公式 X 投稿を出典として引いているが、
**移行後の証明書の扱いやプロキシへの影響は書かれていない**（それは
[艦これツール Wiki](#b-2-艦これ-ツール-wiki) 側にある）。

**扱いやすさ: ×**

- **自動取得できない。** 本調査で `WebFetch https://x.com/KanColle_STAFF` は
  **HTTP 402 Payment Required** を返した（2026-08-02）。X はログインなしの取得を拒む
- 機械可読ではない。日本語の自然文と画像
- 投稿本文は権利者に帰属する。**引用の範囲を超えて転記しない**（[C-05](../spec/constraints.md)）

**注意点・限界**

- **個別投稿の URL は WebFetch で開けても、タイムラインは開けない。**
  「最近何かあったか」を機械的に調べる手段が本プロジェクトには無い
- 実務上は、[攻略 Wiki の `RecentChanges`](#b-1-艦隊これくしょん--艦これ--攻略-wiki) が
  「更新があったか」の**代理指標**として機能する（更新があれば wiki が即座に動く）
- `TODO(要検証)`: ログイン不要で取得できる代替経路。本調査では見つけられなかった

### A-2. 艦これ公式サイト（DMM）

- URL: https://www.dmm.com/netgame/feature/kancolle.html
- 得られるもの: ゲームの紹介、新規着任制限のお知らせ
- 種別: 公式・一次

**速さ: △** — 実在は確認した（HTTP 200、2026-08-02）が、
**ページ内に日付の記載が無く、更新されたかどうかを外から判定できない。**
そのうえページ自身が最新情報を [@KanColle_STAFF](#a-1-kancolle_staff公式-x) へ誘導している。
**したがって、ここを更新の検知に使う意味はない。**

**精度: ◎ / 扱いやすさ: ×** — 理由は A-1 と同じ。

## B. 有志の検証 wiki

**二次情報だが、計算式については実質的にここが最上流である。**
OSS 実装の多くはこの wiki の検証結果を実装している。

### B-1. 艦隊これくしょん -艦これ- 攻略 Wiki*

- URL: https://wikiwiki.jp/kancolle/
- 得られるもの: **計算式**（制空値・索敵・攻撃力・対空カットインなど）、
  **ゲームのルール**（枠数・時間・上限・海域情報）
- 種別: 二次（有志の検証結果の集約）

**速さ: ◎ — 分単位で更新されている。根拠は `RecentChanges` の実測。**

`https://wikiwiki.jp/kancolle/RecentChanges` を 2026-08-02 に確認したところ、
**最新の更新は同日 17:13:21**（確認時点のわずか前）であった。
上位 20 件のうち大半が現行イベント「反撃！第三十一戦隊の戦い」の
海域ページ・ドロップページ・投票ページであり、**イベント進行中は実時間で更新される。**

| 観測した更新（抜粋） | 日時 |
| --- | --- |
| Visby/コメント1 | 2026-08-02 17:13:21 |
| 反撃！第三十一戦隊の戦い/E3 | 2026-08-02 17:06:19 |
| 出撃ドロップ/反撃！第三十一戦隊の戦いE4_01 | 2026-08-02 17:00:47 |

**精度: ○ — 検証されているが、「仮説」と自称する箇所がある。**

- 集計ページ（ドロップ・投票）は多人数の報告を集約しており、**サンプル数が担保されている**
- 一方で**内部値は観測できないため、wiki 自身が仮説と明記している。**
  実例: [formulas/fighter-power.md](formulas/fighter-power.md) が引用しているとおり、
  「艦載機熟練度」ページは内部熟練度を「**内部熟練度仮説**」と呼び、
  上限 120 は「制空値上昇の式と航空優勢／制空権確保の境界から算出」した推定であるとしている
- **したがって wiki の記述をそのまま「検証済み」と扱ってはならない。**
  ページが自ら付けている但し書きを読み、引き継ぐこと

**扱いやすさ: △ — 参照は可。転載は不可。**

- ライセンス表記: **自由利用ライセンスの表示は無い。**
  トップページに「著作権は権利者様側へ帰属しております」とあり、
  画像は著作権法第 32 条の引用として掲載、**画像の転載・再利用は控えるよう明記**されている
  （確認: 2026-08-02）
- **したがって「そこに書かれている事実」を出典付きで参照するのは可。
  文章・表・画像をそのまま複製するのは不可**（[C-05](../spec/constraints.md)）
- 機械可読ではない。HTML の表を人間が読む前提

**注意点・限界**

- **新実装直後は検証が追いついていない。** イベント開始直後のページは
  「暫定」「要検証」を含む。日付を見ること
- 同じ事実が複数ページに別の表現で書かれていることがある。
  [formulas/fighter-power.md](formulas/fighter-power.md) の
  「熟練度 MAX で +25」と「制空ボーナス +22」は同じ値の別表現であった。
  **食い違いに見えて食い違いでない例がある**
- URL に日本語が入る（`https://wikiwiki.jp/kancolle/航空戦`）。
  自動取得するときは percent-encoding が必要

### B-2. 艦これ ツール Wiki*

- URL: https://wikiwiki.jp/kancolletool/
- 得られるもの: **専用ブラウザ・ツールの対応状況**。
  仕様そのものではなく「**他のツールがどう対応したか**」
- 種別: 二次

**速さ: ○ — トップページは化石だが、実務ページは生きている。**

**この乖離が重要である。** トップページの最終更新は **2015-04-04 19:41:24** であり、
一見すると 10 年以上放置されたサイトに見える。しかし `RecentChanges` を見ると:

| ページ | 最終更新 |
| --- | --- |
| toolsfront | 2026-08-01 23:07:26 |
| **各ブラウザ セキュア(SSL)化対応状況** | 2026-08-01 22:53 |
| official_response | 2025-10-22 06:04 |

**トップページの更新日だけを見て「死んでいる」と判断してはならない。**

**精度: ○** — 各ツールの対応状況という検証しやすい事実を扱っており、
利用者からの報告で更新される。仕様そのものは扱っていない。

**扱いやすさ: △** — ライセンス表記なし。B-1 と同じ扱いとする。

**なぜ本プロジェクトに有用か**

「各ブラウザ セキュア(SSL)化対応状況」（2026-08-01 更新）は、
[api/overview.md](api/overview.md) が記録している 2025-10-17 の HTTPS 移行に対して
**各ツールが実際にどう対応したか**を一覧化している。確認した内容の抜粋:

| 状態 | ツール |
| --- | --- |
| 対応済 | Chrome / Firefox、提督業も忙しい!、Electronic Observer 5.3.16 以降、poi 11.0.0、甲ブラウザ、GotoBrowser、航海日誌拡張版 v2.5.7.63 以降、航海日誌改、**KancolleSniffer v16.0** |
| 非対応 | 【赤仮】航海日誌拡張版、KanColleDaemon（Mac） |
| 部分的 | iOS ブラウザ各種（デスクトップ表示でプレイ可。iOS26 で非稼働の報告あり） |

**「KancolleSniffer v16.0」という記述が、[C-6](#c-6-kancollesnifferbitbucket-が現行) の発見の端緒になった。**
GitHub の `fujieda/KancolleSniffer` は v12.11（2021 年）で止まっているためである。

## C. OSS 実装

**動いているツールのコードは、検証を兼ねている。**
間違っていればユーザーが気づき、Issue が立つ。**その意味で wiki より強い場合がある。**

ただし**実装は「その実装者の解釈」でもある。**
観測できない内部値については実装ごとに仮定が異なり、結果が食い違う
（→ [食い違いが起きやすい領域](#食い違いが起きやすい領域)）。

**原則として 2 つ以上の独立実装に当たること。**

### C-1. ElectronicObserver 本家（andanteyk）

- URL: https://github.com/andanteyk/ElectronicObserver
- ライセンス: **MIT**（`gh api` の `spdx_id` で確認、2026-08-02）
- 得られるもの:
  - `ElectronicObserver/Other/Information/apilist.txt`（140 KB）
    — **API のフィールド名と意味の網羅的な一覧**
  - `ElectronicObserver/Other/Information/kcmemo.md`（131 KB）
    — **ゲーム仕様の雑多なメモ**（家具価格、ケッコン後の耐久、パラメータ算出式など）

**速さ: × — この 2 ファイルは 5 年前で凍結している。**

**ここが本調査で最も重要な発見である。** ファイル単位で最終コミット日を確認した。

| 対象 | 最終更新 | 確認方法 |
| --- | --- | --- |
| リポジトリ全体（`pushed_at`） | 2023-10-05 | `gh api repos/andanteyk/ElectronicObserver` |
| **`apilist.txt`** | **2021-09-19** | `gh api "repos/.../commits?path=...apilist.txt&per_page=1"` |
| **`kcmemo.md`** | **2021-04-01** | 同上 |

**リポジトリ全体の `pushed_at` を見て「2023 年まで動いていた」と判断すると誤る。**
参照したいファイルは 2021 年で止まっている。

**精度: ○（ただし出典側が明示的に警告している）**

`apilist.txt` の冒頭 6 行を実際に読んだ。原文は次のとおり（2026-08-02 確認）。

```
艦これ APIリスト
2014/08夏イベント～のAPIの情報を参照
未記入は不明か使いどころなし
推測・噂で書いてある点も多々ある、信じすぎないこと
基本的に戦闘APIはapi_req_sortie/battleを参照すること。他の戦闘系APIの説明は情報が古い可能性がある、差分だけチェックすること
```

**「推測・噂で書いてある点も多々ある、信じすぎないこと」は必ず引き継ぐこと。**
この但し書きを落として `apilist.txt` を出典にした記述は、確度を偽装したことになる。
また「他の戦闘系 API の説明は情報が古い可能性がある」という自己申告もある。

**扱いやすさ: ○** — MIT。ただし `apilist.txt` はタブ区切りの独自フォーマットであり、
機械可読とは言いがたい（冒頭が「タブ幅 4 推奨」と述べている）。

**注意点・限界**

- **2021 年以降に追加されたエンドポイント・フィールドは載っていない**
- それでも**フィールド名の網羅性では現時点で最良**である。
  型定義（[C-2](#c-2-electronicobserveren現行版) / [C-4](#c-4-kagamichankcsapits)）は
  構造を教えるが、「このフィールドが何を意味するか」の日本語の説明は apilist.txt にしかない
- **使い方: 構造は型定義で、意味は apilist.txt で引く。逆にしない**

### C-2. ElectronicObserverEN（現行版）

- URL: https://github.com/ElectronicObserverEN/ElectronicObserver
  （`gre4bee/ElectronicObserver` はここへリダイレクトする）
- ライセンス: **MIT。ただし `gh api` の `spdx_id` は `NOASSERTION` を返す。**
  `LICENSE` の本文を読んで確認した（2026-08-02）。
  逐語の MIT 本文に、次の 2 行の著作権表示が加わっているだけである。

  ```
  Original work copyright (c) 2014 Andante
  Translation work copyright (c) 2015 Ryuu Kitsune
  ```

  **`NOASSERTION` を見て諦めず、本文を読むこと。**
  参照する場合はこの 2 行を帰属表示として引き継ぐ（[C-07](../spec/constraints.md)）。
- 得られるもの:
  - **`ElectronicObserver.KancolleApi.Types/`** — **C# の型定義。
    エンドポイントごとにディレクトリが分かれた、現行の API 構造**
  - `ElectronicObserver/Utility/Data/Calculator.cs` ほか — 計算式の実装
  - `ElectronicObserver.TestData/` — テストデータ

**速さ: ◎ — 本家と対照的に、現在も開発が続いている。**

| 対象 | 日付 |
| --- | --- |
| リポジトリの `pushed_at` | 2026-07-28 |
| 最新リリース `5.4.1` | 2026-06-06 |
| `ElectronicObserver.KancolleApi.Types/` の最終コミット | **2026-05-30** |

**ただし例外がある。** このリポジトリにも
`ElectronicObserver/Other/Information/apilist.txt` と `kcmemo.md` が存在するが、
**最終コミット日は本家と 1 秒も違わない**（それぞれ 2021-09-19 / 2021-04-01）。
**本家からのコピーであり、こちらでも更新されていない。**

> **したがって「EOEN は新しいから apilist.txt も新しい」は誤りである。**
> 現行の構造を知りたいなら `KancolleApi.Types/` を見ること。

型定義が置かれているディレクトリ（2026-08-02 時点で存在を確認）:

`ApiDmmPayment` / `ApiGetMember` / `ApiPort` / `ApiReqAirCorps` / `ApiReqBattleMidnight` /
`ApiReqCombinedBattle` / `ApiReqFurniture` / `ApiReqHensei` / `ApiReqHokyu` / `ApiReqKaisou` /
`ApiReqKousyou` / `ApiReqMap` / `ApiReqMember` / `ApiReqMission` / `ApiReqNyukyo` /
`ApiReqPractice` / `ApiReqQuest` / `ApiReqRanking` / `ApiReqSortie` / `ApiStart2`

**[api/overview.md](api/overview.md) が「未観測」としている出撃・戦闘・建造・遠征系は、
ここに型がある。**

**精度: ○** — 実際に配布され使われているアプリの型であり、
構造が違えばパースに失敗して Issue が立つ。ただし
[C-3](#c-3-kc3kaikc3kai) と計算式が食い違う箇所が既知である
（→ [formulas/fighter-power.md](formulas/fighter-power.md) の「既知の例外」）。

**扱いやすさ: ◎** — C# の型定義であり、構造が機械的に読める。
本家の `apilist.txt` より扱いやすい。

### C-3. KC3Kai/KC3Kai

- URL: https://github.com/KC3Kai/KC3Kai
- ライセンス: **MIT**（`spdx_id` で確認、2026-08-02）
- 得られるもの: **計算式の実装**（`src/library/objects/Gear.js`、
  `src/library/managers/CalculatorManager.js`）、
  **敵艦のデータ**（マスタに無い搭載数など）、`src/library/modules/Network.js`（通信の扱い）

**速さ: ◎ — 本調査で確認した中で最も速い OSS のひとつ。**

| 対象 | 日付 |
| --- | --- |
| `pushed_at` | **2026-08-02**（確認当日） |
| 最新リリース `35.7.3` | 2026-07-22 |
| `src/library/objects/Gear.js`（制空値の実装） | 2026-07-21 |

直近のコミットメッセージが、対応の速さを直接示している（2026-08-02 確認）。

```
2026-07-22  Update abyssal stats, speedgroup, TL pointer
2026-07-21  Update WhoCallsTheFleet ships, partial abyssal stats
2026-07-19  feat: add debuff indicator dot to event sortie boxes
2026-07-18  Update abyssal slots,
```

**深海棲艦のステータスと搭載数を継続的に更新している。**
これは他の情報源から得にくいデータであり、KC3Kai の固有の価値である。

**精度: ○（ただし独自判断が混じる）**

- 星 498、リリースが月に複数回。**間違いはユーザーが見つける規模で使われている**
- 一方で [formulas/fighter-power.md](formulas/fighter-power.md) が記録しているとおり、
  **KC3Kai にしか無い改修係数がある**（大型飛行艇 `0.15 × ★`、master id 486/487 の `0.3`）。
  ソースのコメントは 2023 年と 2025 年の検証ツイートを参照している。
  **新しい検証を先に取り込んだのか、KC3Kai 固有の判断なのかは判別できない**

**扱いやすさ: ○** — MIT。JavaScript のため型定義ほど機械的には読めないが、
`src/data/` 以下に JSON のデータファイルがある。

**関連リポジトリ**

| リポジトリ | ライセンス | 最終 push | 内容 |
| --- | --- | --- | --- |
| `KC3Kai/kc3-translations` | MIT | 2026-07-31 | 艦名・装備名の対訳データ |
| KC3Kai の GitHub Wiki | — | 2026-06-04 | **利用者向けの手順書。API 仕様は無い**。期待しないこと |

### C-4. KagamiChan/kcsapi.ts

- URL: https://github.com/KagamiChan/kcsapi.ts
- ライセンス: **MIT**（`spdx_id` で確認。`Copyright (c) 2018- Poi contributors.`）
- 得られるもの: **kcsapi の TypeScript 型定義**。
  エンドポイントごとにディレクトリが分かれ、`response.ts` に型がある

**速さ: ○ — 更新されているが、頻度は KC3Kai より低い。**

直近のコミットを確認した（2026-08-02）。

```
2026-06-03  1.260604.0
2026-06-03  feat: update api          ← API 定義の更新
2026-05-08  feat: update type generation
2026-05-07  feat: remodel_slot_recover ← 新エンドポイントの追加
```

**版番号が日付ベース（`1.260604.0`）であり、いつの定義かがバージョンから分かる。**
依存関係の更新だけでなく `feat: update api` という実質的な更新が入っている。

**精度: ○（ただし推論由来である）**

[api/overview.md](api/overview.md) が記録しているとおり、**README によれば
これは Phase 2 のサンプルレスポンスから型を推論して生成したものである。**
**推論元のサンプルがいつのものかは不明**であり、現在のゲームと差がある可能性がある。

- **推論由来の型は「観測されなかったフィールド」を落とす。**
  型に無いことは、存在しないことの証明にならない
- 逆に `api_data: null` のような**失敗時の形が型に入っている**のは有用である
  （[api/overview.md](api/overview.md) が `api_get_member/payitem` で利用している）

**扱いやすさ: ◎ — 本調査で確認した中で最も機械可読である。**
TypeScript の型であり、構造を機械的に辿れる。npm パッケージとしても配布されている。

### C-5. poooi/poi

- URL: https://github.com/poooi/poi （サイト: https://poi.moe 、HTTP 200 で確認）
- ライセンス: **MIT**（`spdx_id` で確認）
- 得られるもの: **通信の受け取り方の実装**（`lib/game-api-broadcaster.ts`）、
  プラグイン機構、戦闘予測（`poooi/plugin-prophet`、MIT、2026-07-18）

**速さ: ○ — コードは動いているが、リリースは止まりぎみ。**

| 対象 | 日付 |
| --- | --- |
| `pushed_at` | 2026-07-31 |
| **最新リリース `v11.1.0`** | **2025-11-06** |

**この乖離に注意する。** 開発は続いているが、**正式リリースは 9 ヶ月前**である。
`master` の内容と配布物は一致しない可能性がある。

**精度: ○** — 星 1784 と本調査で最大。広く使われている実装である。

**扱いやすさ: ○** — MIT。ただし `docs/` にあるのは `babel8-migration.md` のみで、
**API 仕様の文書は無い**。コードを読む必要がある。

### C-6. KancolleSniffer（Bitbucket が現行）

**GitHub を見て「止まっている」と判断してはならない。開発の場所が移っている。**

| | GitHub | **Bitbucket（現行）** |
| --- | --- | --- |
| URL | https://github.com/fujieda/KancolleSniffer | **https://bitbucket.org/kancollesniffer/kancollesniffer** |
| 最終コミット | **2021-06-18**（「バージョン12.11の準備」） | **2026-08-01** |
| 最新版 | v12.11 | **v16.16** |
| 星 / 状態 | 2 | 公開・活発 |

- 配布サイト: https://kancollesniffer.bitbucket.io/ （最終更新 2026-08-01、v16.16）
- ライセンス: **Apache-2.0**（`LICENSE.txt` 本文で確認）。
  `NOTICE.md` の著作権表示:

  ```
  Copyright (C) 2013-2021 Kazuhiro Fujieda <fujieda@users.osdn.me>
  Copyright (C) 2021 hATrayflood <h.rayflood@gmail.com>
  ```

- 得られるもの: 遠征・任務・疲労・修理の判定ロジック、戦闘のダメージ計算

**速さ: ◎ — 現行イベントに当日対応している。根拠はコミットログ。**

Bitbucket API で取得した直近コミット（2026-08-02 確認）:

```
2026-08-01T14:50:23Z  v16.16
2026-08-01T08:08:33Z  Merged in fix-combined-second-brank (pull request #473)
2026-08-01T08:01:26Z  #510 連合編成のまま第二艦隊を空にすると発生するエラーを修正
2026-08-01T03:03:47Z  #509 第三十一戦隊 緊急演習！の編成変更
```

**「第三十一戦隊 緊急演習！」は現行イベントの任務名である**
（[攻略 Wiki](#b-1-艦隊これくしょん--艦これ--攻略-wiki) の更新と一致する）。
**任務の編成条件の変更に、その日のうちに追随している。**

**精度: ○** — 任務の達成判定という「間違えると即座にユーザーが気づく」領域を扱っており、
Issue 番号が 500 番台まで伸びている。フィードバックが機能している。

**扱いやすさ: ○ — ただしライセンスが他と違う。ここが最大の注意点である。**

**Apache-2.0 は MIT ではない。** 本プロジェクト（MIT、[ADR-0011](../adr/0011-license-mit.md)）で
扱う際の差は次のとおり。

| | 可否 |
| --- | --- |
| **事実の参照**（「このツールはこう計算している」と出典付きで書く） | 可 |
| **コードの流用** | **避ける。** `NOTICE` の保持・変更点の明示・特許条項の引き継ぎが必要になり、MIT 単独より義務が重い |

**本文書の他の OSS（MIT）と同じ感覚で扱わないこと。**
流用が必要になった場合は [ADR](../adr/) で判断すること。

**注意点・限界**

- **GitHub 側のリポジトリを出典に書かない。** 5 年前の実装である
- Bitbucket は `gh` が使えない。確認は次のコマンドで行う:

  ```sh
  curl -s "https://api.bitbucket.org/2.0/repositories/kancollesniffer/kancollesniffer/commits/master?pagelen=5"
  curl -s "https://bitbucket.org/kancollesniffer/kancollesniffer/raw/master/NOTICE.md"
  ```

### C-7. Nishisonic/logbook（航海日誌拡張版）

- URL: https://github.com/Nishisonic/logbook
- ライセンス: **MIT。`spdx_id` は `NOASSERTION` だが、`LICENSE.txt` の本文は逐語の MIT。**
  `Copyright (c) 2014-2015 航海日誌拡張版開発者`（2026-08-02 確認）
- 得られるもの: 戦闘のダメージ計算、遠征成否チェック、**正確な疲労タイマー**、
  泊地修理タイマー、演習経験値計算
- 系譜: [@sanae_hirotaka](https://github.com/sanaehirotaka/logbook) の「航海日誌」の派生版

**速さ: △** — `pushed_at` は 2026-01-03。**7 ヶ月前**である。
[C-3](#c-3-kc3kaikc3kai) や [C-6](#c-6-kancollesnifferbitbucket-が現行) ほどではない。
ただし [艦これツール Wiki の SSL 対応表](#b-2-艦これ-ツール-wiki) では
「航海日誌拡張版 v2.5.7.63 以降」が対応済とされており、実用されている。

**精度: ○ / 扱いやすさ: ○** — Java 実装。MIT。

**なぜ有用か** — 疲労（コンディション）や泊地修理といった、
**時間経過の扱い**を実装している。[rules/fatigue.md](rules/fatigue.md) や
[rules/timers.md](rules/timers.md) の裏取りに使える系統の実装である。

### C-8. Nishisonic/aaci-prop

- URL: https://github.com/Nishisonic/aaci-prop
- ライセンス: **MIT**（`spdx_id` で確認）
- 得られるもの: **対空カットインの種別と発動確率**
- 速さ: **○** — `pushed_at` 2026-07-20
- 扱いやすさ: **◎** — npm パッケージ（`aaci-prop`）。
  入力（艦 ID と装備 ID）に対して種別・優先度・固定値・確率を返す**関数として使える**

対空カットインは種別が多く、wiki の表を読み違えやすい領域である。
**実装が 1 つの関数にまとまっているのは、突き合わせの対象として扱いやすい。**

`TODO(要検証)`: この実装が依拠している検証の出典。README からは辿れなかった。

## D. 計算ツール

**計算ツールは「答え合わせ」に使う。式の出典にはしない。**

### D-1. 制空権シミュレータ v2

- URL: https://noro6.github.io/kc-web/ （実在確認: 2026-08-02、タイトル「制空権シミュレータ v2」）
- ソース: https://github.com/noro6/kc-web
- ライセンス: **無し。**
  `gh api repos/noro6/kc-web --jq '.license'` は `null`、
  リポジトリ直下のファイル一覧に `LICENSE` は**存在しない**。
  README は Vue CLI のひな型のままで、権利に関する記述が無い（2026-08-02 確認）

> **したがって、このリポジトリのソースコードを参照・流用してはならない。**
> [`kcwikizh/kcdata`](#f-1-kcwikizhkcdata) と同じ扱いである。

- 得られるもの: **制空値・道中の撃墜を含むシミュレーション結果**。
  艦隊編成の共有、統計情報
- 速さ: ○ — `pushed_at` 2026-07-27
- 精度: ○ — 日本語コミュニティで広く使われている

**許される使い方 / 許されない使い方**

| | 可否 |
| --- | --- |
| ツールとして開き、**自分の計算結果と突き合わせる** | 可 |
| **「このツールではこの値になった」と観測として記録する** | 可 |
| ソースコードを読んで式を写す | **不可**（ライセンス無し） |

**注意点**

- **共有 URL の形式が変わっている。** かつて `aircalc.page.link/XXXX` の形で
  共有 URL が発行されていたが、Google の Firebase Dynamic Links 終了に伴い
  **2025-08-25 に無効化された**（作者の告知による）。
  **古い記事や Issue に貼られた `aircalc.page.link` のリンクは、もう開けない**
- v1 にあたる `noro6/kcTools` は **2022-04-23 で停止**しており、
  こちらもライセンス表記が無い。**使わない**

## E. 歴史的資料

**現在の仕様の出典にはならない。「昔はこうだった」を確かめるためだけに使う。**

### E-1. 艦これAPIを叩く

- URL: https://np-complete.gitbook.io/c86-kancolle-api/
- 種別: 同人誌由来の解説（C86 = 2014 年夏コミケ）。ライセンス表記なし
- 速さ: **×**（12 年前）／ 精度: **△**（当時としては詳細）

**なぜ残す価値があるか** — [api/overview.md](api/overview.md) は
`api_get_master/` の存否を `TODO(要検証)` としている。この資料は次を明記している
（2026-08-02 確認）。

> マスタデータのAPIは全て `/kcsapi/api_get_master/` というURLの下にあります

そして**この資料には `api_start2` への言及が無い。**
2014 年時点ではマスタデータが `api_get_master/` 配下にあり、
`api_start2/getData` への集約はその後に起きたことを示す傍証になる。

**ただし、これは「統合された」ことの証明ではない。**
統合の経緯を示す一次資料は依然として見つかっていない（`TODO(要検証)`）。

### E-2. 艦これの API 一覧 Gist（shobotch）

- URL: https://gist.github.com/shobotch/7399829
- タイトル: 「艦これのAPIをひと通り洗ってみました」
- 最終更新: **2018-08-18**（ページ上の表示）
- ライセンス: 表記なし（Gist）
- 内容: `api_get_master/` 配下として `stype` / `furniture` / `slotitem` / `useitem` /
  `maparea` の 5 エンドポイントを列挙している

E-1 と独立に `api_get_master/` の存在を裏付ける。**2 つの独立した資料で確認できた。**

## F. 参照してはいけないもの

**ライセンスが無いソフトウェアは、既定で「無断使用禁止」である。**
「公開されている＝自由に使える」ではない。

### F-1. kcwikizh/kcdata

- URL: https://github.com/kcwikizh/kcdata
- **ライセンス: 無し。** `gh api repos/kcwikizh/kcdata --jq '.license'` が `null`（2026-08-02）
- 状態: 生きている（`pushed_at` 2026-07-29、
  gh-pages の https://kcwikizh.github.io/kcdata/ は HTTP 200）

> **参照も流用も不可。** これは
> [research-kancolle スキル](../../.claude/skills/research-kancolle/SKILL.md)に
> 明文で書かれている禁止事項である。
> **「生きているし内容も良さそうだから」で例外にしない。**

### F-2. noro6/kc-web と noro6/kcTools

→ [D-1](#d-1-制空権シミュレータ-v2)。**ソースは参照不可。ツールとしての利用は可。**

### F-3. kcjervis/jervis（作戦室 Jervis OR）

- URL: https://github.com/kcjervis/jervis
- **ライセンス: 無し**（`spdx_id` = `NONE`）
- 状態: **2023-03-04 で停止**

ライセンスが無く、かつ止まっている。**両方の理由で使わない。**

### F-4. まとめ

| リポジトリ | ライセンス | 状態 | 判断 |
| --- | --- | --- | --- |
| `kcwikizh/kcdata` | **無し** | 生きている | **参照不可** |
| `noro6/kc-web` | **無し** | 生きている | **ソース参照不可**。ツール利用は可 |
| `noro6/kcTools` | **無し** | 2022 で停止 | 使わない |
| `kcjervis/jervis` | **無し** | 2023 で停止 | 使わない |

## G. 止まっている・消えている情報源

**探す手間を省くために書き残す。** 検索で上位に出てくるが、辿っても無駄なものである。

### G-1. 止まっているが、まだ価値があるもの

| 情報源 | 止まった時点 | それでも使える理由 |
| --- | --- | --- |
| `apilist.txt` / `kcmemo.md` | 2021 | **フィールド名の日本語の説明は他に代替が無い**。→ [C-1](#c-1-electronicobserver-本家andanteyk) |
| `fujieda/KancolleSniffer`（GitHub） | 2021-06-18 | 使わない。**現行は Bitbucket**。→ [C-6](#c-6-kancollesnifferbitbucket-が現行) |
| `TeamFleet/WhoCallsTheFleet-DB` | **2024-03-07 にアーカイブ済** | MIT。KC3Kai が「Update WhoCallsTheFleet ships」というコミットを続けており、**KC3Kai 側に取り込まれた形で生きている**。原典より [C-3](#c-3-kc3kaikc3kai) を見るべき |
| 艦これツール Wiki のトップページ | 2015-04-04 | **トップだけが古い。実務ページは 2026-08-01 に更新されている**。→ [B-2](#b-2-艦これ-ツール-wiki) |

### G-2. 消えているもの

| 情報源 | 状態 | 根拠 |
| --- | --- | --- |
| **`aircalc.page.link/*`（制空権シミュレータの共有 URL）** | **無効** | Google の Firebase Dynamic Links 終了に伴い 2025-08-25 に廃止（作者の告知）。古い記事のリンクは開けない |
| **`db.kcwiki.moe`（統計データベース）** | **到達不能** | `curl` が HTTP コードを返さず接続に失敗（2026-08-02） |
| **艦これ検証部の検証 DB** | **停止** | 事実上解散し、検証 DB は稼働していないとされる。`TODO(要検証)`: 二次情報（まとめ Wiki・ニコニコ大百科）でしか確認できていない |

> **「艦これ検証部」と「検証勢」は別物である。**
> 前者は解散した特定の団体、後者は通常のプレイで検証を行うプレイヤー全般を指す。
> [攻略 Wiki](#b-1-艦隊これくしょん--艦これ--攻略-wiki) の検証は後者によるものであり、
> 検証部の解散とは無関係である。**混同して wiki の信頼性を割り引かないこと。**

### G-3. 確認できなかったもの

| 情報源 | 何が起きたか |
| --- | --- |
| https://x.com/KanColle_STAFF | **HTTP 402 Payment Required.** タイムラインを取得できない |
| https://en.kancollewiki.net/ | **HTTP 403 Forbidden.** 内容・鮮度・ライセンスのいずれも確認できていない。`TODO(要検証)` |

**確認できなかったものは推薦しない。** 上記 2 件は「存在は確認したが評価できていない」状態である。

### G-4. 三次情報（攻略ブログ）

| 情報源 | 評価 |
| --- | --- |
| ぜかましねっと艦これ！（https://zekamashi.net/） | **速さ ◎**（最新記事 2026-08-02、現行イベント E5-4 の装甲破砕）。**精度 △**。運営者自身のプレイ経験と、他所の情報のまとめが混在する |

**攻略ブログは「何が起きているか」を素早く知るには有用だが、
計算式やゲームのルールの出典にはしない。**
出典として書くなら、そのブログが参照している元の検証まで辿ること。
辿れないなら `TODO(要検証)` を付ける。

## 食い違いが起きやすい領域

**資料どうしが食い違うのは日常である**
（[research-kancolle](../../.claude/skills/research-kancolle/SKILL.md) 手順 4）。
**あらかじめ「ここは食い違う」と分かっていれば、片方だけ見て断定する事故を防げる。**

### 1. 観測できない内部値 — 最大の食い違い要因

**API から取れない値は、実装ごとに違う仮定が置かれる。必ず食い違う。**

代表例が**内部熟練度**である。API から得られるのは熟練度 0〜7 の段階だけで、
内部熟練度（0〜120）は取得できない。

| 実装 | 採り方 | 熟練度 +7 の内部熟練ボーナス |
| --- | --- | --- |
| ElectronicObserver | 区間の**下限** | √(100/10) = 3.16 |
| KC3Kai | 区間の**中央値** | √(110/10) = 3.32 |

**これは既知の問題として実際に Issue になっている。**

> [KC3Kai/KC3Kai#2194](https://github.com/KC3Kai/KC3Kai/issues/2194)
> “Air power is different from Poi, EO, KCV, etc.”
> — 作成 2017-08-26 / **クローズ 2017-08-30** / コメント 4 件（2026-08-02 確認）

**クローズされているが、食い違い自体は解消していない。**
[formulas/fighter-power.md](formulas/fighter-power.md) が記録しているとおり、
**「正しい 1 つの制空値」は存在しない。**

同種の問題:

- **敵艦の搭載数** — マスタから取れない。KC3Kai は自前のデータで補っている
- **装備ボーナス** — どの計算にどう乗るかが実装ごとに異なる

### 2. 端数処理の位置

`floor` / `ceil` をどこに入れるかで、境界の 1 が変わる。

- 検証 wiki は**倍率の不等号**で書く（「3 倍以上」）
- KC3Kai は**整数の閾値**に落として計算する（`floor(敵/3)` / `ceil(敵×1.5)`）

**同じ式のつもりで、境界だけ結果が違う。** 実測で確かめないと決着しない。

### 3. 同じ値の別表現を「食い違い」と誤認する

[formulas/fighter-power.md](formulas/fighter-power.md) の実例:

- 攻略 Wiki のあるページ: 「熟練度 MAX で艦戦は +25」
- 別のページと OSS 実装: 「制空ボーナス +22」

**これは矛盾ではない。** 22 + √(120/10) ≒ 25.5 であり、
前者は内部熟練ボーナスを含んだ合計値である。
**食い違いを見つけたら、まず定義の範囲が同じかを疑うこと。**

### 4. 新実装直後

イベント開始直後や新装備の実装直後は、**検証が追いついていない。**

- 攻略 Wiki のページに「暫定」「要検証」が付く
- OSS はまず動かすことを優先し、数値の精査は後になる
  （[C-3](#c-3-kc3kaikc3kai) の “partial abyssal stats” というコミットが典型）

**この時期に調べたことは、必ず `TODO(要検証)` を付けて日付を残す。**

### 5. 古い資料と新しい資料

`apilist.txt`（2021）と `KancolleApi.Types`（2026）が食い違ったら、
**原則として新しいほうを採る**。ただし
**「古いほうにしか説明が無い」ケースは多い**（→ [C-1](#c-1-electronicobserver-本家andanteyk)）。
構造は新しい資料、意味は古い資料、という使い分けになる。

## この文書の鮮度管理

**情報源は消える。止まる。引っ越す。**
本調査だけで、消滅 2 件・停止 4 件・**引っ越し 1 件**が見つかった。
**この文書自体が古くなることを前提に運用する。**

### 更新の目安

| きっかけ | やること |
| --- | --- |
| **前回の確認から 3 ヶ月以上経った** | 全エントリの生死を再確認し、冒頭の「最終確認」を更新する |
| 調べ物の途中でリンク切れに当たった | **その場でこの文書を直す。** 後回しにしない |
| 新しい情報源を使った | この文書に追記する（[評価の軸](#評価の軸)の 3 軸で評価すること） |
| ゲームの大きな更新があった | 各 OSS が追随したかを確認する |

### 確認手順

```sh
# 1. GitHub の OSS — ライセンス・アーカイブ状態・最終 push
for r in andanteyk/ElectronicObserver ElectronicObserverEN/ElectronicObserver \
         KC3Kai/KC3Kai poooi/poi KagamiChan/kcsapi.ts \
         Nishisonic/logbook Nishisonic/aaci-prop kcwikizh/kcdata noro6/kc-web; do
  echo "=== $r ==="
  gh api "repos/$r" --jq '{license: (.license.spdx_id // "NONE"), archived, pushed_at}'
done

# 2. 参照したいファイル単位の鮮度（リポジトリ全体の pushed_at では不十分）
gh api "repos/andanteyk/ElectronicObserver/commits?path=ElectronicObserver/Other/Information/apilist.txt&per_page=1" \
  --jq '.[0].commit.committer.date'

# 3. KancolleSniffer は Bitbucket（gh が使えない）
curl -s "https://api.bitbucket.org/2.0/repositories/kancollesniffer/kancollesniffer/commits/master?pagelen=3"

# 4. wiki の生死は RecentChanges を見る（トップページの更新日は当てにならない）
#    https://wikiwiki.jp/kancolle/RecentChanges
#    https://wikiwiki.jp/kancolletool/RecentChanges
```

### 確認するときの原則

1. **リポジトリの `pushed_at` を鮮度と読み替えない。** 参照するファイルの最終コミットを見る
2. **`spdx_id` が `NOASSERTION` でも諦めない。** `LICENSE` の本文を読む
3. **`spdx_id` が `NONE` なら参照しない。** 例外を作らない
4. **リリース日と `pushed_at` の両方を見る。** poi のように 9 ヶ月開くことがある
5. **wiki はトップページではなく `RecentChanges` を見る**

## 確認できなかったこと

**空欄より、誤った断定のほうが害が大きい**
（[research-kancolle](../../.claude/skills/research-kancolle/SKILL.md) 手順 5）。

- `TODO(要検証)`: **@KanColle_STAFF のタイムラインを取得する手段。**
  HTTP 402 で拒否される。個別投稿の URL が既知であれば開けるが、
  「最近何があったか」を機械的に調べる手段が無い
- `TODO(要検証)`: **`en.kancollewiki.net` の内容・鮮度・ライセンス。**
  HTTP 403 で確認できていない。英語圏の主要 wiki と思われるが、**評価していないので推薦しない**
- `TODO(要検証)`: **艦これ検証部の検証 DB が停止した経緯と時期。**
  二次情報（まとめ Wiki・ニコニコ大百科）でしか確認できていない
- `TODO(要検証)`: **`Nishisonic/aaci-prop` が依拠している検証の出典。**
  README からは辿れなかった
- `TODO(要検証)`: **`api_get_master/` が `api_start2/getData` に統合された経緯。**
  [E-1](#e-1-艦これapiを叩く) / [E-2](#e-2-艦これの-api-一覧-gistshobotch) により
  2014〜2018 年時点で `api_get_master/` が存在したことは 2 つの独立した資料で確認できたが、
  **統合の時期と経緯を示す一次資料は見つかっていない**
- `TODO(要検証)`: **中国語圏の情報源の評価。** `kcwikizh` 系は存在を確認したが、
  中核の `kcdata` が**ライセンス無しのため参照不可**であり、
  周辺リポジトリの評価まで踏み込んでいない
- `TODO(要検証)`: **統計データベース系の代替。** `db.kcwiki.moe` は到達不能。
  ドロップ率などの統計を機械可読な形で得る手段は、本調査では見つけられなかった
  （[攻略 Wiki](#b-1-艦隊これくしょん--艦これ--攻略-wiki) の集計ページは人間向けである）
