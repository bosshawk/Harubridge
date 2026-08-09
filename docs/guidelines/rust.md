# ガイドライン: Rust の書き方

> **ステータス: Draft（承認待ち）。**
> `docs/guidelines/` の変更には人間の承認が必要。

- 適用範囲: `crates/` 配下のすべての Rust コード
- ルール ID の略号: `RS`
  [0008](../adr/0008-code-as-source-of-truth.md)（コードが正）/
  [0018](../adr/0018-dependencies.md)（`thiserror` と `anyhow`）/
  [0022](../adr/0022-observed-data-privacy.md)（観測データとフィクスチャ）/
  [0025](../adr/0025-clock-handling.md)（時刻）/
  [0027](../adr/archive/0027-repository-layout.md)・[0030](../adr/archive/0030-no-named-architecture.md)（**構造。本文書は触れない**）
- ステータス: ドラフト（承認待ち）

## 原則

**このガイドラインは「Rust をどう書くか」だけを扱う。**
どこに置くかは [architecture.md](../spec/architecture.md) とリポジトリの実体、何と名付けるかは
[glossary.md](../spec/glossary.md) が正である。ここでは繰り返さない。

判断の拠り所は 2 つ。

1. **デファクトに従えば済むものはルール化しない。**
   [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)・`rustfmt` の既定・
   `clippy` の既定に従えばよいことは、ここに書かない。
   **ここに書いてあるルールは、デファクトだけでは決まらないもの**である。
2. **人間の常時レビューが無い**。
   したがって「気をつける」で守るルールは書かない。
   コンパイラ・`clippy`・CI のいずれかが落とせる形にするか、
   落とせないなら**なぜ落とせないかを添える**。

そのうえで、この文書の重心は **[NFR-003](../spec/requirements.md)（未知の構造でも停止せず縮退）**と
**[C-04](../spec/constraints.md) / [C-07](../spec/constraints.md)（観測データを外へ出さない）**にある。
本アプリが依存する艦これの API は非公開仕様であり（[C-03](../spec/constraints.md)）、
**「壊れたら落ちる」書き方は、この 1 点だけで不適格になる。**

## ルール

ID は `G-RS-<連番>`。連番は主題ごとに桁を分けてある（命名 01〜 / エラー処理 10〜 /
縮退 20〜 / panic 30〜 / ログ 40〜 / テスト 50〜 / clippy 60〜 / コメント 70〜）。
**一度振った ID は再利用しない。**

### 命名

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-01 | MUST | 表記は Rust API Guidelines の [C-CASE](https://rust-lang.github.io/api-guidelines/naming.html) に従う。型・trait・列挙子は `UpperCamelCase`、値・関数・モジュールは `snake_case`、定数と `static` は `SCREAMING_SNAKE_CASE` |
| G-RS-02 | MUST | 略語は 1 つの単語として扱う（`Hp` / `Id` / `Api` / `Los`）。大文字を連ねない（[glossary.md 原則 4](../spec/glossary.md#原則-4-略語は-1-つの単語として扱う)） |
| G-RS-03 | MUST | 艦これのゲーム概念に付ける英語識別子は [glossary.md](../spec/glossary.md) を正とする。**ここで訳語を決めない。** 用語集に無い語が必要になったら、**用語集に追加してから使う**（`docs/spec/` は人間の承認が要る階層） |
| G-RS-04 | MUST | 型名は単数、集合を持つ変数は複数形、辞書は `<値>_by_<鍵>`（[glossary.md](../spec/glossary.md) の原則 5） |
| G-RS-05 | MUST | 艦これの API の語（`api_*` / `deck` / `slotitem` / `cond`）を受信モジュールの外へ出さない（[glossary.md 原則 2](../spec/glossary.md#原則-2-艦これ-api-の語を識別子に持ち込まない)）。これは**入れ子の private モジュールでコンパイラが強制する** |
| G-RS-06 | SHOULD | 型の変換は `From` / `TryFrom` の実装として書く。専用の `convert_*` 関数を作らない（変換の実体は `From` / `TryFrom` に統一する） |

### エラー処理

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-10 | MUST | ライブラリ的な層（パース・変換・計算・永続化）は `thiserror` でエラーを型として定義する。アプリの端（Tauri コマンド・タスクの最上段・`main`）で `anyhow` に集約する |
| G-RS-11 | MUST NOT | `anyhow::Error` / `anyhow::Result` を `harubridge-core` の公開 API の戻り値や構造体のフィールドに置かない。**型で分岐できなくなると NFR-003 の縮退が書けない** |
| G-RS-12 | SHOULD | エラーの列挙子は「**呼び出し側が分岐する単位**」で切る。分岐しないものは 1 つにまとめる。逆に、縮退の分かれ目になるもの（未知のフィールド / 型の不一致 / 未知の ID）は必ず別の列挙子にする |
| G-RS-13 | MUST | `#[from]` は**文脈を足す必要がないとき**だけ使う。どの API パスの・どの位置で失敗したかを持たせる必要があるなら、`#[from]` ではなくフィールドを持つ列挙子にして `map_err` で包む（構造サマリはこの文脈から組み立てる） |
| G-RS-14 | MUST | `?` は「**呼び出し側がその失敗ごと捨ててよい**」ところまでで止める。要素単位の縮退が要る場所（G-RS-23）では `?` を使わない |
| G-RS-15 | MUST NOT | エラーを `to_string()` した文字列で分岐しない。分岐が要るなら列挙子を足す |

### 未知のデータへの縮退（NFR-003 / C-03）

**この節がこのガイドラインの中心である。**
「ワイヤ型」とは、艦これの JSON を `serde` で直接受ける型を指す
（受信モジュール内の private な「外部モデル」）。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-20 | MUST | ワイヤ型のフィールドは**すべて `Option<T>`** にし、構造体に `#[serde(default)]` を付ける。「必ず来るはず」のフィールドを非 `Option` にしない。**非公開仕様に「必ず」は無い**（[C-03](../spec/constraints.md)） |
| G-RS-21 | MUST NOT | ワイヤ型に `#[serde(deny_unknown_fields)]` を付けない。serde の既定は**自己記述的な形式では未知のフィールドを黙って無視する**（[serde 公式](https://serde.rs/container-attrs.html)）。それでよい。**危険なのはフィールドの追加ではなく、削除と型変更のほう**であり、そちらは G-RS-20 が受け止める |
| G-RS-22 | MUST | ワイヤ型は `#[serde(flatten)] extra: HashMap<String, serde_json::Value>` の受け皿を持ち、**`extra` が空でなければログに出す。これがゲーム更新の検知になる。** ただし出すのは**キー名だけで、値は出さない**（[C-04](../spec/constraints.md)） |
| G-RS-23 | MUST | 配列は**要素ごとに変換し、失敗した要素だけ落として残りを通す。** `collect::<Result<Vec<_>, _>>()?` でレスポンス全体を捨てない。外部仕様が「解釈できた行だけを表示する」（[timers.md E-03](../spec/external/timers.md)）「一部の項目が欠けても行は表示する」（[fleet-view.md E-04](../spec/external/fleet-view.md)）と約束しているため |
| G-RS-24 | MUST | 得られなかった項目は `Option` のまま内部モデルへ運ぶ。`0` / `""` / `-1` で埋めない。UI が `—` を出せなくなる（[fleet-view.md E-04](../spec/external/fleet-view.md)） |
| G-RS-25 | MUST | マスタに無い ID（未知の艦娘・装備・艦種）は**エラーにしない。** ID を保持したまま内部モデルへ通し、名前を解決できなかったことを型で表す（[fleet-view.md E-03](../spec/external/fleet-view.md) の `不明 (ID: 1234)`） |
| G-RS-26 | MUST | 縮退したときは**必ず記録を残す。** 黙って捨てない。記録の中身は G-RS-42 に従う |

### `panic` / `unwrap` / `expect`

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-30 | MUST NOT | 観測の経路（受信・パース・変換・状態更新・永続化・計算）に `unwrap()` / `expect()` / `panic!` / `todo!` / `unimplemented!` / スライスの添字アクセス / `Option::take` 前提の前提置きを書かない。**入力は艦これが決めるのであって、我々の不変条件ではない** |
| G-RS-31 | MAY | `#[cfg(test)]` と `tests/`、`build.rs`、および `main` の起動時（設定の読み込み失敗など、続行しても意味が無い箇所）では `expect` を使ってよい。メッセージには「**何が成り立つはずだったか**」を書く。`"unwrap failed"` と書かない |
| G-RS-32 | MUST | 数値演算は `checked_*` / `saturating_*` を使う。特に残り時間は `saturating_sub` で 0 止めする（[timers.md E-04](../spec/external/timers.md) が「負の値を表示しない」と約束している） |
| G-RS-33 | MUST | 観測ループを `tokio::spawn` するときは `JoinHandle` を捨てない。**panic はランタイムを落とさず、そのタスクだけを終わらせる**ため、放置すると「アプリは動いているのに観測だけ止まっている」状態になる（[tokio 公式](https://docs.rs/tokio/latest/tokio/task/struct.JoinError.html)。`JoinError::is_panic` で判別できる）。この状態は [fleet-view.md E-02](../spec/external/fleet-view.md)（最終観測時刻の表示）でユーザーに見える形にする |

### ログとマスキング（C-04 / C-07）

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-40 | MUST NOT | 観測に由来する値を `tracing` のフィールドや `{:?}` にそのまま渡さない。**`#[derive(Debug)]` した構造体を `tracing::debug!(?resp)` で出すと、提督名（`api_nickname`）・`api_member_id`・自由記述（`api_comment`）が丸ごとログに落ちる。** 実データで確認済みの項目である |
| G-RS-41 | MUST | ワイヤ型と、観測データを保持する内部モデルには **`Debug` を derive しない。** 必要なら `Debug` を手で実装し、**構造だけ**（型名・件数・ID・キー名）を出す。`derive` は「誰かがうっかり `{:?}` する」経路を作ってしまう |
| G-RS-42 | MUST | ログに出してよいのは**構造サマリの範囲**に限る。API パス・失敗の種類・失敗位置の JSON パス・その位置の型名と長さ・兄弟キーのキー名。キー名は `^api_[a-z0-9_]+$` に一致するものだけそのまま出し、一致しないものは長さと文字種に落とす |
| G-RS-43 | MUST NOT | リクエストの URL・クエリ文字列・ボディをログに出さない。`api_token` が入る（[C-04](../spec/constraints.md)） |
| G-RS-44 | SHOULD | レベルの目安。未知のキーの検出（G-RS-22）と縮退の発生（G-RS-26）は `warn`。**これは異常ではなく「ゲームが更新された可能性がある」という通知**であり、埋もれさせない。通常の観測は `trace` |

### テスト

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-50 | MUST NOT | 実測した `/kcsapi/` のレスポンスをテストコード・フィクスチャ・スナップショットに入れない。**一度コミットすれば履歴から消せない**（[C-07](../spec/constraints.md)）。フィクスチャは生成器の出力のみ |
| G-RS-51 | MUST | ワイヤ型と変換のテストは、**同じモジュールの中の `#[cfg(test)] mod tests`** に書く。これらは private な入れ子モジュールであり、**`tests/` からは到達できない**（`tests/` は外部クレートとして public API しか見えない）。`tests/` に置くのは、クレートの公開 API を通した結合テストだけ |
| G-RS-52 | MUST | 縮退のテストは最低 3 種類を持つ。**(a) フィールドが消えた / (b) 型が変わった（数値のはずが文字列・`null`）/ (c) 未知のキーが増えた。** (c) は `extra` に入ることを検証する |
| G-RS-53 | SHOULD | 実測データを読むテストは `#[ignore]` を付け、git 管理外のディレクトリ（`.local/`）を読む。CI では走らない |
| G-RS-54 | MUST | テストで現在時刻を暗黙に読まない。時刻は引数で渡す。永続化のテストは実際のアプリデータ領域ではなく一時ディレクトリを使う |

### `clippy` と静的解析

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-60 | MUST | `cargo fmt --check` と `cargo clippy --all-targets -- -D warnings` が通ること（警告をエラー扱い） |
| G-RS-61 | MUST | 「この関数を使わない」という規律は、文章ではなく `clippy.toml` の `disallowed-methods` / `disallowed-types` に書く。**書ける規律を文章に留めない**（`clippy.toml` は `CARGO_MANIFEST_DIR` から親へ遡って探索されるため、クレートごとに置ける） |
| G-RS-62 | MUST | `#[allow(...)]` は**最小のスコープ**に付け、**必ず理由をコメントで添える。** クレート全体（`#![allow(...)]`）やファイル先頭に付けない |

### コメントとドキュメントコメント

「コードが正」の原則の下では、
**コメントは「コードから読み取れないもの」だけを書く場所**になる。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-RS-70 | MUST NOT | コードを読めば分かることを書かない。書くのは「**なぜそうしたか**」と「**何を知っていないと危ないか**」 |
| G-RS-71 | MUST | **計算式には出典コメントを必ず書く**（[NFR-009](../spec/requirements.md)）。参照した OSS 名・ファイル・URL・参照日、または `docs/kancolle/formulas/` への参照。出典が無い式には `TODO(要検証)` を付ける |
| G-RS-72 | MUST | 艦これの非公開仕様に依存する箇所（フィールド名・列挙値・枠数）には、`docs/kancolle/` の該当文書へのリンクを書く（[C-03](../spec/constraints.md)） |
| G-RS-73 | SHOULD | `pub` な項目には doc コメントを書く。ただし**署名で分かることは書かない。** 書くのは前提条件・失敗する条件・**単位**（ミリ秒か秒か、0 始まりか 1 始まりか） |
| G-RS-74 | MUST NOT | doc コメントに層構造・モジュールの割り方・ディレクトリ構成の説明を書かない。**同じ説明が 2 箇所にあると必ず片方が古くなる** |
| G-RS-75 | MUST NOT | コメント・テスト名・doc の例に、実測の提督名・`api_member_id`・`api_token` を書かない（[C-07](../spec/constraints.md)） |

## 具体例

抽象的なルールは守られない。以下はすべて**同じ 1 本のレスポンス処理**を題材にしている。

### 1. ワイヤ型の定義（G-RS-20 / G-RS-21 / G-RS-22）

#### 良い例

```rust
/// 艦これ `api_port/port` の `api_ship` の要素。
/// 観測: docs/kancolle/api/api_port_port.md（2026-08-02）
#[derive(Deserialize)]           // Debug は derive しない（G-RS-41）
#[serde(default)]
struct ApiShip {
    api_id: Option<i64>,
    api_ship_id: Option<i64>,
    api_lv: Option<i64>,
    api_nowhp: Option<i64>,
    api_maxhp: Option<i64>,

    /// 上で受けきれなかったキーの受け皿。空でなければゲームが更新された可能性がある。
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
```

#### 悪い例

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApiShip {
    api_id: i64,
    api_ship_id: i64,
    api_lv: i64,
    api_nowhp: i64,
    api_maxhp: i64,
}
```

なぜ悪いのか:

- `deny_unknown_fields` は**フィールドが 1 つ増えただけでレスポンス全体を失敗させる。**
  艦これの API は予告なく変わる（[C-03](../spec/constraints.md)）。
  しかも serde の既定は未知のフィールドを黙って無視するので、**放っておけば正しく動く。**
  自分から壊れやすくしている
- **本当に危ないのは削除と型変更のほう**（`api_nowhp` が消える / `null` になる / 文字列になる）で、
  `deny_unknown_fields` はそれを 1 つも防がない。防ぐのは `Option` + `#[serde(default)]` である
- 未知のキーを捨てているので、**ゲームが更新されたことに誰も気づけない**
- `Debug` を derive しているため、`tracing::debug!(?ship)` の 1 行で
  観測データがログへ落ちる経路ができている（G-RS-41）

### 2. 未知のキーの検出（G-RS-22 / G-RS-42 / G-RS-44）

#### 良い例

```rust
if !wire.extra.is_empty() {
    // キー名だけを出す。値は出さない（C-04）
    let keys: Vec<&str> = wire.extra.keys().map(String::as_str).collect();
    tracing::warn!(api = "api_port/port", ?keys, "未知のフィールドを検出した");
}
```

#### 悪い例

```rust
if !wire.extra.is_empty() {
    tracing::warn!("未知のフィールド: {:?}", wire.extra);   // 値ごと出ている
}
```

なぜ悪いのか: `extra` は `HashMap<String, Value>` であり、
**`{:?}` は値をそのまま文字列化する。** 増えたフィールドが自由記述や識別子だった場合、
それがログファイルに残る。
poi の実データで確認されたとおり、`api_comment`（提督の自由記述）のようなフィールドは実在する。
**「増えたフィールドが安全かどうか」は、増えてみるまで分からない。**

### 3. 要素ごとの縮退（G-RS-14 / G-RS-23 / G-RS-26）

#### 良い例

```rust
let mut ships = Vec::with_capacity(wire.api_ship.len());
for (index, w) in wire.api_ship.into_iter().enumerate() {
    match Ship::try_from(w) {
        Ok(ship) => ships.push(ship),
        Err(e) => tracing::warn!(api = "api_port/port", index, error = %e, "1 隻を落として続行する"),
    }
}
```

#### 悪い例

```rust
let ships: Vec<Ship> = wire
    .api_ship
    .into_iter()
    .map(Ship::try_from)
    .collect::<Result<Vec<_>, _>>()?;   // 1 隻失敗すると全滅する
```

なぜ悪いのか: `collect::<Result<Vec<_>, _>>()` は**最初の失敗で打ち切り、それまでの成功分も捨てる。**
`?` でさらに関数ごと抜けるため、**300 隻中 1 隻の解釈に失敗しただけで艦隊パネルが空になる。**
[fleet-view.md](../spec/external/fleet-view.md) の E-03 / E-04 は
「他の列と他のパネルの表示を続ける」「行自体は表示する」と約束しており、これはその約束を破る。

### 4. エラー型の粒度と `#[from]`（G-RS-10 / G-RS-12 / G-RS-13）

#### 良い例

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// 呼び出し側が「この API を丸ごと諦める」と判断する材料になる
    #[error("JSON として読めない: {0}")]
    Malformed(#[from] serde_json::Error),

    /// 呼び出し側が「この要素だけ落とす」と判断する材料になる
    #[error("{path} が欠けている")]
    MissingField { path: String },

    #[error("{path} の型が想定と異なる（{found}）")]
    UnexpectedType { path: String, found: &'static str },
}
```

#### 悪い例

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("パースに失敗した: {0}")]
    Failed(#[from] anyhow::Error),
}
```

なぜ悪いのか:

- 列挙子が 1 つしかないので、**呼び出し側が「全体を諦める」か「要素だけ落とす」かを判断できない。**
  結果として全部を諦める実装になり、NFR-003 の縮退が成立しない
- `anyhow::Error` を型の内側に置いている（G-RS-11）。
  `anyhow` は「もう分岐しない」ことを表明する道具であり、**分岐する層に持ち込むと型が意味を失う**
- 失敗位置（`path`）を持たないため、
  構造サマリを組み立てられない。**縮退したが原因が分からない**という一番困る状態になる

### 5. `Debug` の手書き（G-RS-40 / G-RS-41）

#### 良い例

```rust
impl std::fmt::Debug for Admiral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 提督名・ID は出さない（C-04 / C-07）
        f.debug_struct("Admiral").field("hq_level", &self.hq_level).finish_non_exhaustive()
    }
}
```

#### 悪い例

```rust
#[derive(Debug)]
pub struct Admiral {
    pub name: String,       // 提督名
    pub member_id: i64,
    pub hq_level: u8,
}
```

なぜ悪いのか: この型が**どこか 1 箇所で `{:?}` に渡された瞬間**、
提督名と `api_member_id` がログに残る。ログは
**ユーザーから辿れる場所**に置かれ（[NFR-008](../spec/requirements.md)）、不具合報告に添付されうる。
**`derive(Debug)` は「安全な既定」ではなく、「全フィールドを出力する実装を自動生成する」ものである。**
[C-07](../spec/constraints.md) の下では、その出力先が公開の Issue になりうる。

### 6. 出典コメント（G-RS-71 / G-RS-72）

#### 良い例

```rust
/// 制空値を計算する。
///
/// 出典: docs/kancolle/formulas/fighter-power.md
/// （KC3改 `src/library/objects/Gear.js` の `fighterVeteran` / 2026-08-02 参照）
/// TODO(要検証): 内部熟練度は参照元自身が「仮説」と明記している
fn fighter_power(gears: &[Gear]) -> u32 {
```

#### 悪い例

```rust
/// 制空値を計算する
fn fighter_power(gears: &[Gear]) -> u32 {
    // 対空 * sqrt(搭載数) + 熟練度ボーナス
```

なぜ悪いのか: **この式がどこから来たのかが分からない。**
艦これの計算式は公式に公開されておらず（[C-03](../spec/constraints.md)）、
すべて第三者の検証結果である。出典が無いと、**式が間違っていたときに何を疑えばよいか分からず、
ゲーム側の仕様変更なのか写し間違いなのかも区別できない。**
[NFR-009](../spec/requirements.md) が「計算式には出典を必ず併記する」と定めているのはこのためである。
コメントの `// 対空 * sqrt(...)` はコードを言い換えただけで、G-RS-70 にも反する。

## 例外

**MUST は原則として破らない。** ただし次の 3 つは、条件付きで例外を認める。

| ルール | 破ってよい条件 | 書き残すもの |
| --- | --- | --- |
| G-RS-30（`unwrap` / `expect` 禁止） | **観測の経路の外**であり、失敗したら続行しても意味が無いとき（`main` の初期化・`build.rs`・テスト。G-RS-31） | `expect` のメッセージに「何が成り立つはずだったか」 |
| G-RS-41（`Debug` を derive しない） | その型が**観測に由来する値を 1 つも持たない**とき（ID だけの型・列挙・設定の列挙値など） | 型の doc コメントに「観測データを持たない」ことを 1 行 |
| G-RS-62（`#[allow]`） | `clippy` の指摘が誤検出であるとき | `#[allow]` の直上に**誤検出である理由**。「うるさいから」は理由ではない |

**次の 2 つには例外を認めない。**

- **G-RS-50**（実測データをコミットしない）。[C-07](../spec/constraints.md) により取り返しがつかない
- **G-RS-43**（URL をログに出さない）。`api_token` の混入は事故として回復できない

例外を使うときは、**ルール ID をコメントに書く**（`// G-RS-41 の例外: 観測データを持たない`）。
レビューで grep できるようにするためであり、
エージェントが実装するこの体制では**検索できない合意は存在しないのと同じ**である。

## 未解決事項

- `TODO(未確定)`: **本文書は人間の承認を受けていない**（冒頭のステータス）。
  `docs/guidelines/` は承認が要る階層である。
- `TODO(要検証)`: **Tauri のコマンドハンドラ内で panic したときの挙動。**
  G-RS-33 で確認したのは `tokio::spawn` したタスクの挙動
  （[tokio 公式](https://docs.rs/tokio/latest/tokio/task/struct.JoinError.html)）までで、
  `#[tauri::command]` の中の panic が
  プロセスを落とすのか・その呼び出しだけが失敗するのかは実測していない。
  **G-RS-30 の根拠の強さがここで変わる**ため、実装着手時に確かめる。
- `TODO(要検証)`: `clippy.toml` で禁止する具体的な項目の一覧。
  現時点で確定しているのは `std::time::Instant::now` の 1 件だけで、
  `disallowed-methods` に `serde_json::from_str` の直接呼び出しなどを足すかは未決。
  **`clippy` の `restriction` グループ（`unwrap_used` / `expect_used` / `indexing_slicing` など）を
  どこまで有効にするかも含めて、Cargo.toml の `[lints]` 表を作る時点で決める。**
- `TODO(未確定)`: 構造サマリ（G-RS-42）を組み立てる実装の形。
  「構造サマリに残す情報の粒度」も未決のまま同じ論点に含まれる。
- `TODO(未確定)`: `serde` のワイヤ型を**手で書くか、マクロで畳むか。**
  G-RS-20 / G-RS-22 は全ワイヤ型に同じ 2 行を要求するため、
  忘れを機械的に検出したくなる。ただし**マクロは読み解きの面を増やす**ため、
  最初は手で書き、痛くなってから考える。
