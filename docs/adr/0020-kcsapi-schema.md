# ADR-0020: kcsapi の構造は Rust の型定義を正とし、JSON Schema を生成物として出力する

- ステータス: Proposed
- 日付: 2026-08-02
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0008](0008-code-as-source-of-truth.md), [ADR-0011](0011-license-mit.md),
  [ADR-0016](0016-tech-stack.md), [ADR-0018](0018-dependencies.md),
  [docs/spec/constraints.md](../spec/constraints.md)（C-02 / C-03 / C-05 / C-07）,
  [docs/spec/architecture.md](../spec/architecture.md)

## 背景と課題

艦これの API（kcsapi）の仕様は公式に公開されておらず、本アプリが依存する構造は
すべて観測に基づく推定である（[C-03](../spec/constraints.md)）。
プロジェクトオーナーから「OpenAPI.yaml などの形で残したい」という要望が出ている。

しかし [ADR-0008](0008-code-as-source-of-truth.md) は
「個別 API のパース処理は文書化しない。コードが正」と定めている。
スキーマ文書を人手で書けば、Rust の型定義との**二重管理**が発生し、
ADR-0008 が避けようとした状態そのものになる。この緊張を解く必要がある。

一方で、コードだけでは足りないものもある。C-03 は
「API 構造を書くときは根拠（実測日 / 参照した OSS）を必ず併記する」と要求しており、
**いつ観測した構造なのか**と**前回から何が変わったか**はコードからは読み取れない。
非公開仕様は予告なく変わるため、差分を検知する基準となる成果物には価値がある。

前提として次の制約が効く。

- [C-02](../spec/constraints.md): 通信は読み取り専用。**こちらからリクエストを送らない**
- [C-03](../spec/constraints.md) / NFR-003: 未知のフィールドや構造変化に遭遇しても停止せず縮退する
  （[requirements.md](../spec/requirements.md)）
- [C-05](../spec/constraints.md) / [C-07](../spec/constraints.md): ゲーム由来のデータを同梱しない。
  実測ログを資料にするときは識別情報をマスクする
- [ADR-0011](0011-license-mit.md): 本プロジェクトは MIT。GPL 系の資産は流用できない

### 調査した事実

**kcsapi の形**（実測。2026-08-02。[architecture.md](../spec/architecture.md#外部との接点)）

- リクエストは POST の form-encoded
- レスポンスは純粋な JSON ではなく、先頭に `svdata=` が付いた文字列である

**先行資産**

| 資産 | ライセンス | 形式 | 最終更新 | 出典 |
| --- | --- | --- | --- | --- |
| `KagamiChan/kcsapi.ts` | **MIT** | TypeScript 型 **＋ JSON Schema (draft-06)** | 2026-06-03 | [GitHub](https://github.com/KagamiChan/kcsapi.ts) / [npm](https://www.npmjs.com/package/kcsapi) |
| `andanteyk/ElectronicObserver` の `Other/Information/` | MIT | `apilist.txt`（140KB）/ `kcmemo.md`（131KB）の**散文** | 2023-10-05 | [GitHub](https://github.com/andanteyk/ElectronicObserver/tree/master/ElectronicObserver/Other/Information) |
| `poooi/poi` | MIT | （スキーマ資産の有無は未評価） | 2026-07-31 | [GitHub](https://github.com/poooi/poi) |
| `KC3Kai/KC3Kai` | MIT | （同上） | 2026-08-01 | [GitHub](https://github.com/KC3Kai/KC3Kai) |
| `kcwikizh/kcdata` | **無し** | — | 2026-07-29 | [GitHub](https://github.com/kcwikizh/kcdata) |

確認した範囲に GPL 系は無かった。ライセンス無しの `kcdata` は流用できない。

`kcsapi.ts` は最も有用である（2026-08-02 時点の GitHub API / npm registry API による実測）。

- **108 エンドポイント**について `request.ts` / `response.ts` と
  `request.json` / `response.json`（JSON Schema draft-06）を持つ
- 生成経路は「匿名化サンプル JSON → [quicktype](https://github.com/glideapps/quicktype) → 型と JSON Schema」。
  サンプルは文字列を `"Tanaka"`、数値を `0` に置換して匿名化されている（[README](https://github.com/KagamiChan/kcsapi.ts/blob/master/README.md)）
- 更新は継続している（最新公開 `1.260604.0` / 2026-06-03。DL は約 500/月と小さい）
- **限界**: サンプルからの推論のため、`api_get_member/basic` の応答スキーマは
  33 プロパティすべてが `required`、かつ `additionalProperties: false` である
  （[response.json](https://github.com/KagamiChan/kcsapi.ts/blob/master/api_get_member/basic/response.json)）。
  また `api_result` / `api_data` の外側の封筒は剥がされており、対象外である。
  ランキング系は暗号化のため除外されている（README）

**コード生成ツール**（crates.io / GitHub API による実測。2026-08-02）

| ツール | 向き | 版 | 規模 | ライセンス |
| --- | --- | --- | --- | --- |
| [`schemars`](https://github.com/GREsau/schemars) | **Rust 型 → JSON Schema (2020-12)** | 1.2.2 | 直近 90 日 1.31 億 DL | MIT |
| [`typify`](https://github.com/oxidecomputer/typify) | JSON Schema → Rust 型 | 0.7.0 | 直近 90 日 360 万 DL | Apache-2.0 |
| [`progenitor`](https://github.com/oxidecomputer/progenitor) | OpenAPI → Rust **クライアント** | 0.14.0 | 直近 90 日 133 万 DL | MPL-2.0 |
| [`openapi-generator`](https://github.com/OpenAPITools/openapi-generator) | OpenAPI → 各言語の**クライアント/サーバ** | — | ★26,640 | Apache-2.0 |
| [`quicktype`](https://github.com/glideapps/quicktype) | JSON / JSON Schema → Rust ほか | — | ★13,813 | Apache-2.0 |

`schemars` は serde の属性を尊重して schema を生成する（[README](https://github.com/GREsau/schemars/blob/master/README.md)）。
`typify` は自ら「work in progress」と明記し、`anyOf` の扱いが不正確であることなどを
限界として挙げている（[README](https://github.com/oxidecomputer/typify/blob/main/README.md)）。

**OpenAPI で `svdata=` を表現できるか**

OpenAPI 3.1 は JSON Schema 2020-12 に準拠しており、文字列の中身を
`contentMediaType` / `contentSchema` で**注釈**できる。
しかし JSON Schema 2020-12 はこれらを assertion ではなく annotation と定義し、
「実装は既定で文字列の中身をデコード・パース・検証してはならない」と規定している
（[JSON Schema Validation 2020-12 §8](https://json-schema.org/draft/2020-12/json-schema-validation)）。
つまり `svdata=` の内側の構造は**書けるが効かない**。検証もコード生成も届かない。

## 決定

**Rust の型定義（serde の構造体）を kcsapi 構造の唯一の正とする。**
そこから `schemars` で JSON Schema を**機械生成**し、リポジトリにコミットされる
生成物として残す。**スキーマを人手で編集しない。OpenAPI 文書は作らない。**

- 生成方向は **コード → スキーマ**の一方向に固定する。逆向きのコード生成は行わない
- 生成物は CI で再生成し、差分があれば失敗させる（コミット漏れを構造的に防ぐ）
- 型を起こす際の参照資産として `KagamiChan/kcsapi.ts` の JSON Schema（MIT）を用いる。
  **取り込むのではなく参照する。** MIT の帰属表示を行う（[C-07](../spec/constraints.md)）
- 観測日と参照元は Rust の型に doc コメントとして併記する（[C-03](../spec/constraints.md)）

これは [ADR-0008](0008-code-as-source-of-truth.md) の例外ではない。
ADR-0008 が却下したのは「**人が読む内部仕様書**をコードから生成すること」であり、
本 ADR が作るのは人が読み替えるための文書ではなく、
**非公開仕様の変化を差分で検知するための機械可読な成果物**である。
コードが正である点は変わらない。

## 検討した選択肢

### 案 A: Rust の型を正とし、`schemars` で JSON Schema を生成する（採用）

- 概要: 上記のとおり。
- 利点: 正が 1 つしかないため二重管理が原理的に起きない。
  `schemars` は serde 属性を尊重するため、生成されたスキーマは
  実際のデシリアライズ結果と一致する。採用規模が大きく（直近 90 日 1.31 億 DL）、MIT。
  スキーマをコミットすることで、ゲーム側の構造変化が git の差分として見える。
- 欠点: 依存が 1 つ増える。`serde` / `specta` に加えて `JsonSchema` の derive が並ぶ。
  また、まだ実装していない API は当然スキーマにも現れない
  —— 記録は「本アプリが解釈している範囲」に限られ、kcsapi 全体の網羅記録にはならない。

### 案 B: OpenAPI.yaml を人手で書いて正とし、そこから Rust を生成する

- 概要: オーナー要望どおり OpenAPI 文書を作り、`openapi-generator` か `progenitor` で
  Rust 側を生成する。
- 利点: 成果物が標準形式であり、閲覧ツールが豊富。エンドポイント一覧が一望できる。
- 欠点: `svdata=` を表現できない（上記のとおり注釈止まり）。
  form-encoded リクエストは表現できるが、本アプリはリクエストを組み立てない。
- 却下理由: OpenAPI の Rust 生成器は**リクエストを送るクライアント**を出力するものであり、
  [C-02](../spec/constraints.md)（読み取り専用・リクエストを追加しない）に反する成果物になる。

### 案 C: JSON Schema を正とし、`typify` で Rust の型を生成する

- 概要: `kcsapi.ts` の JSON Schema を取り込む、あるいは自前で JSON Schema を書き、
  `typify` で Rust の型を生成する。
- 利点: 108 エンドポイント分の既存スキーマを初日から使える。MIT で流用可能。
- 欠点: `typify` 自身が work in progress を自認しており、`anyOf` の扱いが不正確。
- 却下理由: サンプル推論のスキーマは全プロパティが `required` かつ
  `additionalProperties: false` であり、そこから生成した型は未知フィールドや欠落フィールドで
  即座に失敗する —— NFR-003 / [C-03](../spec/constraints.md) の縮退動作と正面から衝突する。

### 案 D: `kcsapi.ts` を npm 依存として取り込み、TypeScript 側で型を使う

- 概要: `npm i kcsapi` し、フロントエンドで kcsapi のレスポンス型を使う。
- 却下理由: [architecture.md](../spec/architecture.md) では UI は Rust コアから受け取った状態のみを
  描画し、kcsapi の生レスポンスに触れない。型が効く場所が存在しない。

### 案 E: 何も残さず、ADR-0008 のまま Rust のコードだけを正とする

- 概要: 現状維持。スキーマ成果物を作らない。
- 却下理由: 観測時点と変化の差分がどこにも残らず、
  [C-03](../spec/constraints.md) が要求する「根拠の併記」を運用で担保できない。

## 決め手

**機械可読な記録を得るために、「スキーマを人が書く」ことを捨てた。**
生成の向きをコード → スキーマの一方向に固定した結果、
残したいもの（差分で追える構造の記録）だけが手に入り、二重管理は発生しない。

## 影響

- 実装への影響:
  - `schemars` を Rust の依存に追加する（[ADR-0018](0018-dependencies.md) の一覧に 1 行増える）
  - パース層の型に `#[derive(JsonSchema)]` が加わる。
    NFR-003 を守るため、型は**未知フィールドを拒否せず、欠落を許容する**設計を維持する
    （`deny_unknown_fields` を使わない、`Option` と既定値を活用する）
  - スキーマ生成と差分検査を CI に追加する
- ドキュメントへの影響:
  - [ADR-0018](0018-dependencies.md) の本文は書き換えない（[ADR の不変性](README.md#ルール)）。
    依存の追加は本 ADR が根拠となる
  - [architecture.md](../spec/architecture.md) の「非公開仕様への依存を局所化する境界」に
    生成物の位置づけを 1 〜 2 行追記する必要がある（**人間の承認が必要**）
  - `kcsapi.ts` を参照した場合の MIT 帰属表示を、`THIRD-PARTY` 相当の記載に加える
- 取り消す場合のコスト: **低。** 生成物を捨て、derive を外すだけで済む。
  Rust の型定義は正であり続けるため、失われるものが無い

## 未解決事項

- `TODO(未確定)`: 生成物の置き場所とファイル分割（`schema/kcsapi/*.json` か 1 ファイルか）。
  最初の実装で決める
- `TODO(未確定)`: 匿名化したサンプル JSON をリポジトリに置くかどうか。
  `kcsapi.ts` は置いているが、[C-05](../spec/constraints.md) / [C-07](../spec/constraints.md) との
  兼ね合いを別途判断する
- `TODO(未確定)`: 未評価 —— `schemars` の `JsonSchema` derive と
  `specta` / `tauri-specta`（[ADR-0018](0018-dependencies.md)）の `Type` derive を
  同一の型に併用したときの相性。実装時に確認する
- `TODO(未確定)`: 未評価 —— `poooi/poi` と `KC3Kai/KC3Kai` が持つ kcsapi 関連資産の中身。
  ライセンスが MIT であることのみ確認し、スキーマ相当の資産があるかは調べていない
- `TODO(未確定)`: 未評価 —— `quicktype` の Rust 出力の品質。
  Rust ターゲットが存在することのみ確認した
- `TODO(未確定)`: 未評価 —— `ElectronicObserver` の `apilist.txt` / `kcmemo.md` の網羅範囲。
  散文であり機械可読でないことは確認したが、内容の突き合わせは行っていない
