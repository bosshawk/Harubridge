# ADR-0030: アーキテクチャを名乗らず、関心事ごとの素朴な分割に留める

- ステータス: Superseded by [ADR-0032](0032-repository-structure.md)（決定は変更なしで統合された）
- 日付: 2026-08-08
- 決定者: プロジェクトオーナー（承認待ち）
- 関連: [ADR-0003](0003-agent-driven-development.md)（実装は AI エージェントが行う）/
  [ADR-0008](0008-code-as-source-of-truth.md)（コードが正）/
  [ADR-0016](0016-tech-stack.md) / [ADR-0024](0024-state-sync-granularity.md) /
  [ADR-0027](0027-repository-layout.md)（**外側の構成。本 ADR は内側を扱う**）/
  [ADR-0028](0028-quest-counter-schema.md)

## 背景と課題

[ADR-0027](0027-repository-layout.md) はディレクトリ構成を決めたが、
**その内側をどういう構造で書くか**は未定だった。

DDD / オニオンアーキテクチャ / ヘキサゴナル（ポート＆アダプタ）/ クリーンアーキテクチャ /
Feature-Sliced Design といった既成の型を採るかどうかは、
**後から変えるコストが高く、かつ事情を知らない人が必ず再提案する**論点である。

あわせてオーナーから方針が示された。

> 基本的に全ての対応については**一般的なデファクトスタンダードを採用したい**

## 決定

**アーキテクチャの名前を名乗らない。** Rust 側・TypeScript 側とも、
**関心事ごとの素朴な分割**に留め、既成の型の語彙（レイヤー名・ポート・アダプタ・
ユースケース・エンティティ・スライス）を持ち込まない。

**強制される境界は次の 3 本だけとし、それ以外は規約ではなく「置き場所の目安」とする。**

| 境界 | 強制する主体 |
| --- | --- |
| Rust: コア（`crates/harubridge-core/`）は `tauri` を知らない | **コンパイラ**（クレート分割。[ADR-0027](0027-repository-layout.md)）|
| Rust: **艦これの API の語彙（`api_*`）は受信モジュールの外へ出ない** | **コンパイラ**（入れ子の private モジュール。下記）|
| TS: `features/` どうしが直接 import しない。依存は `shared → features → app` の一方向 | **リンタ**（`eslint-plugin-import` の `import/no-restricted-paths`）|

### 腐敗防止層をコンパイラに守らせる

[glossary.md](../spec/glossary.md) の原則 2 と
[architecture.md](../spec/architecture.md)（非公開仕様への依存をパース層に閉じ込める）は、
**規約ではなく型検査にできる。**

Rust の可視性は「private な項目は、そのモジュールと**その子孫**からのみ見える」と定義されている
（Rust Reference / Visibility and Privacy）。したがって:

- 受信モジュールの**中に** `mod wire;` を **private で**宣言すると、
  艦これの API の型は受信モジュールとその子孫からしか見えない
- **兄弟モジュール（状態・永続化・UI へ出す形）からは到達できず、コンパイルが通らない**

平らに並べるとこれは成立しない。同じクレート内のモジュールは互いに `pub(crate)` で見えるためである。
**入れ子にすることが強制力を生む。**

> **実測で確認済み**（rustc 1.94.1、2026-08-08）。
> 兄弟からの参照は `error[E0603]: module 'wire' is private` で停止し、
> 子孫からの参照はコンパイルが通る。

**クレートを増やさずに済む**ため、[ADR-0027](0027-repository-layout.md) の
「境界をクレートに昇格させるのは、そこに機械的に強制したい規範があるときに限る」と両立する。

### 補助的な強制（クレートを増やさずに効くもの）

`clippy.toml` は `CARGO_MANIFEST_DIR` から親へ遡って探索されるため、
**クレートごとに別の設定を置ける。** [ADR-0018](0018-dependencies.md) が
「clippy の警告をエラー扱い」と決めているので、そのまま CI で落とせる。

```toml
# crates/harubridge-core/clippy.toml
disallowed-methods = [
    { path = "std::time::Instant::now" },
]
```

これで [ADR-0025](0025-clock-handling.md) の「単調増加時計を使わない」を機械的に守れる。
`TODO(要検証)`: ワークスペースルートにも `clippy.toml` がある場合の優先順位は未実測。

### Rust 側

`crates/harubridge-core/` の内側は、関心事ごとの module 分割に留める。

| module | 責務 |
| --- | --- |
| **受信**（この中に private な下位モジュールを持つ）| 艦これ JSON の受け取りと、内部モデルへの変換。**艦これの語彙を知る唯一の場所** |
| ├ 外部モデル（**private**）| `serde` の型。**兄弟から到達できない** |
| └ 変換（**private**）| **腐敗防止層の実体。層ではなく関数の集合として置く** |
| 内部モデル | アプリが表示・計算に使う形。**我々が所有する唯一の形** |
| 状態 | メモリ上の現在状態。[ADR-0024](0024-state-sync-granularity.md) の push 単位と一致させる |
| 永続化 | `state/*.json` / `events/*.jsonl` / `replays/`（[ADR-0021](0021-data-persistence.md)）|
| 任務カウンタ | **唯一の自前ドメインロジック**（[ADR-0028](0028-quest-counter-schema.md)）|
| 計算 | 制空値・索敵値などの純関数。出典コメント必須（NFR-009）|
| **UI へ出す形** | マスタと結合済みの読み取り用。**内部モデルとは別に持つ**（下記）|

**内部モデルと「UI へ出す形」を分ける。** [ADR-0024](0024-state-sync-granularity.md) は
名前解決をRust 側で済ませると決めているが、`state/*.json` は起動時に**マスタ無しで復元される**。
したがって保存する形は ID を持ち、UI へ出す形はマスタと結合した結果を持つ。
**型を 2 組作る理由はこれだけであり、機械的に DTO を並べるためではない。**

**入力の軸と出力の軸は直交する。** 外部モデルと変換は**エンドポイント別**に割り、
内部モデルは**ドメイン別**に割る。`api_port/port` の 1 本が艦隊・保有艦・入渠・資源・
基地航空隊を同時に更新するためである。
**したがって Rust 側に TypeScript 側と同じ機能別の割り方を持ち込まない。**

**module の名前は決めない。コードが正である**（[ADR-0008](0008-code-as-source-of-truth.md)）。
本 ADR が決めるのは「層を名乗らない」ことであって、module 名の一覧ではない。

**腐敗防止層は採用する。** ただし実体は `From` / `TryFrom` の実装であり、
DDD の語彙は使わない。艦これの API の語をどこまで持ち込むかは
[glossary.md](../spec/glossary.md) の原則 2 が定める（パース層に閉じ込める）。

**trait による抽象は、実装が実際に 2 つ以上できるまで入れない。**

### TypeScript 側

```
src/
├── main.tsx
├── bindings.ts          tauri-specta の生成物
├── app/                 features を合成する唯一の場所（情報パネルの配置・通知の常駐）
├── ipc/                 listen 登録と invoke の初期 pull を集約
├── store/               1 ストア + ドメイン単位のスライス
├── features/            外部仕様（docs/spec/external/）と対応
└── shared/              機能をまたぐもののみ
```

- **`invoke` / `listen` を `ipc/` の外に書かない**
- **イベント 1 本 = スライス 1 つ。** 中身は「全量・観測時刻・未観測フラグ」のみとし、
  **マージ処理を書かない**（[ADR-0024](0024-state-sync-granularity.md)。UI の store はキャッシュであり正ではない）
- **残り時間と絞り込み結果をストアに持たない。** 描画時に導出する
- **未仕様の機能のディレクトリを先掘りしない**
- **パネルの外枠を `shared/` に置く。** 最終更新時刻の表示・未観測時の案内（E-01）・
  **パネル単位のエラー境界**（E-02。1 枚が落ちても他のパネルは表示を続ける）は
  全パネルが共有する。13 枚分の縮退実装が 1 箇所に集約される

> **`app/` はファイルではなくディレクトリにする。** 情報パネルの配置（FR-007）は
> 配置の切り替え・境界のドラッグ・次回起動時の復元を伴い、1 ファイルに収まらない。
> また `import/no-restricted-paths` の zone は「`app` だけが `features` を import してよい」を
> **パスで表現する**必要がある。

> **ストアは 1 本にし、ドメインごとに「スライス」で分ける。**
> zustand の公式ドキュメントが "Your applications global state should be located in a
> **single Zustand store**"、大きい場合は "splitting the store into **slices**" と述べている。
> 購読は**ストア単位ではなく selector 単位**であるため、
> 1 本にまとめても「変化していないドメインは再描画しない」は失われない
> （[ADR-0024](0024-state-sync-granularity.md) が「1 本の大きな状態を受け取っても、
> 再描画は selector で絞れる」と既に記録している）。**挙動は同じで、公式推奨に一致する。**

## 検討した選択肢

### 案 A: アーキテクチャを名乗らない（採用）

### 案 B: DDD の戦術パターンを採る

- 却下理由: **守るべき対象が無い。**
  [C-02](../spec/constraints.md) により通信は読み取り専用で、
  **ゲームへ送るコマンドが 1 つも存在しない。**
  集約・不変条件・トランザクション整合性・リポジトリは、いずれも書き込みを前提とする道具である。
  さらに入力は `api_port/port` の**全量スナップショット**（実測 271,824 bytes）で毎回丸ごと差し替わるため、
  **集約のライフサイクルが存在しない。**
  集約を作ると [ADR-0024](0024-state-sync-granularity.md) の
  「全量スナップショットをそのまま流す」を壊す
- **ドメインを所有していない**ことも効く。モデルは艦これ側が決める。
  「ユビキタス言語をドメインエキスパートと作る」という前提が成立しない
  （我々が決められるのは**英語識別子の対応だけ**である。→ [glossary.md](../spec/glossary.md)）
- 我々が所有するドメインロジックは**任務カウンタ 1 つ**で、1 module に収まる

### 案 C: オニオン / クリーンアーキテクチャ

- 却下理由: **中心に置くドメインを所有しておらず、ユースケース層が実質空になる。**
  Rust 側の流れは「観測 → パース → 状態更新 → emit」の 1 本しかない。
  TypeScript 側はさらに徹底しており、[ADR-0024](0024-state-sync-granularity.md) により
  名前解決も計算も縮退も Rust 側で済ませてから渡るため、
  **UI に残るのは「IPC の配線・純関数の書式化・描画」の 3 種類だけ**である。
  UI 側に domain 層を作ると「store はキャッシュであり正ではない」を破る

### 案 D: ヘキサゴナル（ポート＆アダプタ）

- 却下理由: **ポートを立てる境界が実質 1 本（永続化）しかなく、そこは差し替え候補が潰れている。**
  [ADR-0021](0021-data-persistence.md) がプレーンファイルに確定させており、
  外部 I/O は注入スクリプト 1 本（[architecture.md](../spec/architecture.md)）と
  LLM の HTTP 1 本（[ADR-0017](0017-llm-endpoint.md)）だけ。
  **実装が常に 1 つの trait は、間接参照を増やすだけである。**
  テストは永続化を `tempdir` で回すほうが、ADR-0021 の規律を実際に検証できる

### 案 E: Feature-Sliced Design（TypeScript 側）

- 却下理由: **7 レイヤーのうち Pages / Features / Entities / Processes の 4 枚が空になる。**
  ウィンドウは 1 枚でルーティングが無く（FR-001）、ビジネスロジックが UI に無いため。
  **公式自身が「名前が重要」と述べている体系で 4/7 を空にするのは、誤読を招く**

### 案 F: 種類別ディレクトリ（`components/` `hooks/` `stores/`）

- 却下理由: 実測で確認した。**Clash Verge Rev**（Tauri v2 + React）は種類別を採っており、
  **1 機能が 4 ディレクトリに散っていた。**
  [CLAUDE.md](../../CLAUDE.md) §0 が求める「ドキュメントとの対応関係」が構造から読めなくなる

### 案 G: レイヤードを宣言する

- 却下理由: 案 A との差が**呼び方だけ**である。
  しかも任務カウンタ（自前ロジック + 永続化 + 照合）がどの層にも素直に収まらない

## 決め手

### 1. デファクトである

**Rust 側 —— DDD / ヘキサゴナルは Rust のデファクトではない。**

- `language:rust (hexagonal-architecture OR ddd) in:name,description,topics stars:>200` の
  **総ヒットが 5 件**。うち 3 件は Event Sourcing / CQRS の**ライブラリ**、
  1 件は誤ヒット、1 件は自称 example。**アプリケーションが事実上見当たらない**
- Web 検索の上位は個人ブログと Medium のチュートリアルばかりで、
  出てくる「実プロジェクト例」は記事内の教材である
- DI コンテナ `shaku` は 90 日 36,632 DL。
  [ADR-0018](0018-dependencies.md) が採用規模を理由に却下したクレートと同じ桁である
- 逆に、**条件が最も近い実在プロジェクト FUSOU**（Tauri v2 + Rust + 艦これ、MIT）の
  `src-tauri/src/` は `cmd` / `storage` / `window` / `senders` / `notify` で、**層の名前がゼロ**。
  腐敗防止層も `kc-api-dto` → `kc-api-interface-adapter`（`from_trait/`）→ `kc-api-interface` と、
  **Rust 標準の `From` として実装されている**
- **GitButler**（Tauri v2、48 クレート）も全部「能力」名で、
  Tauri クレートは平坦なファイル 12 枚の配線のみ

**TypeScript 側 —— 機能別ディレクトリがデファクトである。**

- **Redux 公式スタイルガイド**が "feature folder" を **Priority B: Strongly Recommended** とし、
  種類別を「older codebases のやり方」と名指しで退けている
- bulletproof-react も同形に加え「機能間 import 禁止」「`shared → features → app` の一方向」を規範化
- **React 公式にはディレクトリの規範が無く**、Vite テンプレートは `src/` に 4 ファイルのみ。
  足場は何も押し付けてこない

### 2. コンパイラとリンタが止められる違反にしか意味がない

[ADR-0016](0016-tech-stack.md) の決め手は
**「人間の常時レビューが無い体制では、コンパイラが最後の防波堤になる」**だった。

Rust 側で 5 つの分割案を比較した結果、
**コンパイラが強制できる境界は [ADR-0027](0027-repository-layout.md) の 1 本だけ**である。
module 境界も trait による層分けも、違反したままビルドが通る。
**コンパイラを味方にしない層分けは、規約を増やすだけで防波堤にならない。**

TypeScript 側で唯一機械的に強制できるのが
`eslint-plugin-import` の `import/no-restricted-paths` であり、**だからこれを入れる**。
（[ADR-0019](0019-linter.md) で `biome` から `eslint` に変えていたことが、ここで効いた。）

### 3. 間接参照は、この体制では直接コストになる

[ADR-0003](0003-agent-driven-development.md) の体制で効くのは「書く手間」ではなく
**「文脈に載る量」**である。層を増やすファイル数の増加は AI にとって安いが、
`trait Repository` → `impl` → DI 配線 と 3 ホップ辿らないと実際の処理に届かない構造は、
**エージェントが読み解きに失敗する面を増やす。**

「人間の生産性」「オンボーディングの速さ」を根拠にした一般論は、ここでは使えない。

## 影響

- [ADR-0027](0027-repository-layout.md) の `src/` の図に `ipc/` と `store/` が加わる
- **`eslint-plugin-import` の追加**が必要（[ADR-0018](0018-dependencies.md) 系列）。
  **これが無いと TypeScript 側の境界は誰も検出できない**
- `docs/guidelines/` は**層構造に触れない。** 本 ADR を参照する
- 取り消す場合のコスト: 中。名乗っていない構造に後から名前を付けるのは、
  名乗った構造から降りるより易しい

## 未解決事項

- `TODO(未確定)`: module 名の確定。**コードが正**（[ADR-0008](0008-code-as-source-of-truth.md)）であり、
  本 ADR では決めない
- `TODO(未確定)`: `shared/` を 1 枚に束ねるか、トップレベルに平置きするか。
  実装着手時に決めてよい粒度
- `TODO(要検証)`: `import/no-restricted-paths` の zone 定義が、
  実際に 3 本で意図どおり効くか（実装着手時に確認する）
- 本 ADR は**構造を決めるが、名前を決めない。**
  「どこに何を置くか迷ったら本 ADR の関心事の表を見る」までが射程である

## 根拠

| 記述 | 根拠 | 参照日 |
| --- | --- | --- |
| FUSOU のモジュール構成と `From` による腐敗防止層 | `tsukasa-u/FUSOU`（MIT）: `packages/FUSOU-APP/src-tauri/src/`、`kc-api-interface-adapter/from_trait/` | 2026-08-08 |
| GitButler の 48 クレートが能力名であること | `gitbutlerapp/gitbutler`: `crates/` | 2026-08-08 |
| Rust の DDD / ヘキサゴナル系リポジトリが 5 件しかないこと | GitHub 検索 `language:rust (hexagonal-architecture OR ddd) in:name,description,topics stars:>200` | 2026-08-08 |
| `shaku` の採用規模 | crates.io 90 日 36,632 DL | 2026-08-08 |
| feature folder が Priority B: Strongly Recommended であること | Redux 公式スタイルガイド | 2026-08-08 |
| 種類別で 1 機能が 4 ディレクトリに散ること | `clash-verge-rev/clash-verge-rev`（Tauri v2 + React）の実測 | 2026-08-08 |
| React 公式にディレクトリの規範が無いこと / Vite テンプレートが `src/` に 4 ファイルであること | React 公式ドキュメント / `create-vite` の react-ts テンプレート | 2026-08-08 |
| `api_port/port` が全量 271,824 bytes であること | [api_port_port.md](../kancolle/api/api_port_port.md)（本プロジェクトの実測） | 2026-08-02 |
