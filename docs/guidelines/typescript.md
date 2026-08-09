# ガイドライン: TypeScript / React の書き方

> **ステータス: Draft（承認待ち）。**
> `docs/guidelines/` の変更には人間の承認が必要（[ADR-0015](../adr/0015-documentation-layout.md)）。

- 適用範囲: `src/` 配下のすべての TypeScript / TSX、および注入スクリプト
- ルール ID の略号: `TS`
- 関連 ADR: ADR-0003 / ADR-0008 / ADR-0016 / ADR-0018 / ADR-0019 / ADR-0024 / ADR-0025 / ADR-0026 / ADR-0027 / ADR-0029 / ADR-0030
- ステータス: ドラフト

## 原則

**UI は薄い。** [ADR-0024](../adr/0024-state-sync-granularity.md) により、真実の情報源は Rust コアにある。
名前解決も計算も縮退の判定も Rust 側で済んでから届くため、
**TypeScript 側に残るのは「IPC の配線」「純関数の書式化」「描画」の 3 種類だけ**である
（[ADR-0030](../adr/archive/0030-no-named-architecture.md) 案 C の却下理由）。
このガイドラインは、**その薄さを保つために書いてはいけないもの**を主に定める。

判断に迷ったら次の 2 つを拠り所にする。

1. **デファクトに従う。** React 公式ドキュメントの推奨と `typescript-eslint` の推奨セットを根拠にする。
   **推奨セットで機械的に検出できるものは、ここでルール化しない**（重複した規約は必ず片方が腐る）。
   独自ルールを置くときは、このプロジェクト固有の理由を必ず添える
2. **コンパイラとリンタが最後の防波堤である。**
   人間が常時レビューしない体制（[ADR-0003](../adr/0003-agent-driven-development.md)）では、
   検査を弱める方向の変更（`any`・`as`・`eslint-disable`）は、それ自体が事故である
   （[ADR-0016](../adr/0016-tech-stack.md) / [ADR-0019](../adr/0019-linter.md) の決め手）。

**ディレクトリ構成と層の話はここに書かない。**
[ADR-0030](../adr/archive/0030-no-named-architecture.md)（`src/` の内側）と
[ADR-0027](../adr/archive/0027-repository-layout.md)（リポジトリ全体）が正である。

## ルール

ID は `G-TS-<連番>`。強度は `MUST`（必須）/ `SHOULD`（原則そうする）/ `MAY`（してもよい）。

### 型の使い方

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-01 | MUST | `any` を書かない。型が定まらない値は `unknown` で受け、絞り込んでから使う |
| G-TS-02 | MUST | 型アサーション（`as`）で検査を黙らせない。`as const` と、DOM API のように絞り込みようがない箇所だけに許し、その場に理由を 1 行書く |
| G-TS-03 | MUST | `@ts-ignore` を使わない。どうしても要るときは `@ts-expect-error` に理由を添える |
| G-TS-04 | MUST | **`bindings.ts`（`tauri-specta` の生成物）を手で編集しない。** 直したいものは Rust 側の型を直す |
| G-TS-05 | MUST | **Rust から届くドメイン型を UI 側で再定義しない。** `bindings.ts` の型を import して使う |
| G-TS-06 | MAY | 表示都合の型（絞り込み条件・並べ替えキー・パネルの配置）は UI 側で定義してよい。これらは Rust に存在しない |
| G-TS-07 | MUST | `tsconfig` の `strict` を切らない。個別ファイルで無効化もしない |

`unknown` で受けるのは**注入スクリプトから来る値と `JSON.parse` の結果だけ**である。
`bindings.ts` を経由する値には既に型が付いているため、そこで `unknown` は出てこない。

**良い例**

```ts
import type { FleetSnapshot } from "../bindings";

// Rust 由来の型はそのまま使う。UI 側で writable にしない
export function sortedShips(fleet: FleetSnapshot): readonly Ship[] { … }
```

**悪い例**

```ts
// ❌ Rust の型を UI 側で writable に書き直している
interface Fleet {
  ships: Ship[];
  updatedAt: number;
}

const fleet = payload as Fleet; // ❌ 検査を黙らせている
```

なぜ悪いのか: **Rust 側の型が変わってもコンパイルが通ってしまう。**
[ADR-0018](../adr/0018-dependencies.md) が `tauri-specta` を選んだのは
「登録したが型を出し忘れたというズレが構造的に起きない」ためであり、
UI 側で型を書き直すとその利点が消える。`as` はさらに、ズレを実行時まで先送りする。

### 命名

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-10 | MUST | **艦これのゲーム概念に自分で英語名を付けない。** [glossary.md](../spec/glossary.md) を引く。無い語は glossary に追加してから使う（承認が要る） |
| G-TS-11 | MUST | **`api_*` 由来の語を `src/` に持ち込まない**（glossary 原則 2 / [ADR-0030](../adr/archive/0030-no-named-architecture.md)）。UI に届く時点で艦これの語彙は消えている |
| G-TS-12 | MUST | 表記は glossary 原則 3・4・5 に従う（型・コンポーネントは `PascalCase`、値・関数は `camelCase`、略語は `Hp` / `Id` / `Lbas`） |
| G-TS-13 | MUST | コンポーネントは `PascalCase`。フックは `use` で始まる `camelCase`（`eslint-plugin-react-hooks` がこの規則で対象を判定する） |
| G-TS-14 | MUST | イベントハンドラの実体は `handleXxx`、props として渡す口は `onXxx`（React 公式 "Responding to Events" の慣習） |
| G-TS-15 | SHOULD | ファイル名は中身に合わせる。コンポーネントを定義するファイルは `PascalCase.tsx`、それ以外は `camelCase.ts` |

**良い例**

```tsx
// ExpeditionPanel.tsx
type Props = { onSelectFleet: (fleetId: FleetId) => void };

export function ExpeditionPanel({ onSelectFleet }: Props) {
  const handleRowClick = (fleetId: FleetId) => onSelectFleet(fleetId);
  …
}
```

**悪い例**

```tsx
// ❌ 艦これ API の語がそのまま UI に出ている
type Props = { api_deck_id: number; onClickDeck: (id: number) => void };
// ❌ glossary に無い自前の英訳
function KanmusuList() { … }
```

なぜ悪いのか: 語の対応を各所で勝手に決めると、**同じ概念に 2 つの名前ができる**
（glossary 原則 6）。`api_*` を持ち込むと、非公開仕様への依存がパース層の外へ漏れ、
艦これ側の変更が UI まで波及する（[architecture.md](../spec/architecture.md)）。

### React の書き方

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-20 | MUST | 関数コンポーネントで書く。**クラスは Error Boundary の実装だけに許す**（React 公式が「関数コンポーネントで Error Boundary を書く方法は現在無い」と明記している） |
| G-TS-21 | MUST | フックはコンポーネント／フックの**トップレベル**でのみ呼ぶ（React 公式 Rules of Hooks）。`eslint-plugin-react-hooks` に検出させる |
| G-TS-22 | MUST | **`useEffect` は外部システムとの同期にのみ使う。** 描画のためのデータ変換とイベントの結果を Effect に書かない（React 公式 "You Might Not Need an Effect"） |
| G-TS-23 | MUST | **既存の state / props から計算できる値を state に持たない。** 描画時に計算する |
| G-TS-24 | MUST | **`setInterval` を個々のコンポーネントに置かない。** 毎秒の tick はアプリ全体で 1 本だけ（[ADR-0024](../adr/0024-state-sync-granularity.md) 決定 6） |
| G-TS-25 | MUST | tick を購読するのは**残り時間を実際に描画するコンポーネントだけ**にする。上位で受けて props で配らない |
| G-TS-26 | MUST | ストアの購読は **selector で必要な最小単位まで絞る。** 引数なしの `useStore()` を書かない |
| G-TS-27 | MUST | selector の中で新しいオブジェクト・配列を作らない。複数の値をまとめて取るときは `useShallow` を使う（[ADR-0024](../adr/0024-state-sync-granularity.md)） |
| G-TS-28 | MUST | リストの `key` に配列 index を使わない。艦娘 ID など**行に固有で安定した値**を使う |

**なぜ毎秒 tick を 1 本に絞るのか。** [external/timers.md](../spec/external/timers.md) は
残り時間を毎秒更新すると定めており、Rust が渡すのは絶対完了時刻だけである
（[ADR-0025](../adr/0025-clock-handling.md)）。
tick を粗い場所で購読すると、**毎秒アプリ全体が再描画される。**
調べた既存実装 3 本（poi / ElectronicObserver / KancolleSniffer）がいずれもタイマーを 1 本に
していることが [ADR-0024](../adr/0024-state-sync-granularity.md) に記録されている。

**良い例**

```tsx
// 残り時間を出すセルだけが tick を購読する
function Countdown({ completesAt }: { completesAt: number }) {
  const now = useNowTick();               // アプリ全体で 1 本の tick
  return <span>{formatDuration(completesAt - now)}</span>;
}

// 一覧は tick を知らない。完了時刻が変わらない限り再描画されない
function ExpeditionRows() {
  const rows = useAppStore(useShallow((s) => s.expedition.rows));
  return rows.map((r) => <ExpeditionRow key={r.fleetId} row={r} />);
}
```

**悪い例**

```tsx
// ❌ パネルが tick を購読し、残り時間を state に持っている
function ExpeditionPanel() {
  const rows = useAppStore((s) => s.expedition.rows);
  const [remaining, setRemaining] = useState<number[]>([]);
  useEffect(() => {
    const id = setInterval(
      () => setRemaining(rows.map((r) => r.completesAt - Date.now())),
      1000,
    );
    return () => clearInterval(id);
  }, [rows]);
  …
}
```

なぜ悪いのか: 3 つ同時に壊している。
(1) 毎秒 `setState` するため**パネル全体が毎秒再描画される**（G-TS-25）。
(2) 残り時間は完了時刻と現在時刻から**計算できる導出値**であり、state に持つと二重管理になる
（G-TS-23 / [ADR-0025](../adr/0025-clock-handling.md) 決定 2）。
(3) タイマーがコンポーネントの数だけ増える（G-TS-24）。

### 状態の扱い

[ADR-0024](../adr/0024-state-sync-granularity.md) の
「**UI の store はキャッシュであり、正ではない**」を守るための節。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-30 | MUST | **`invoke` / `listen` を `ipc/` の外に書かない**（[ADR-0030](../adr/archive/0030-no-named-architecture.md)）。コンポーネントもフックも直接呼ばない |
| G-TS-31 | MUST | **受け取ったペイロードで丸ごと置き換える。マージ・パッチ・差分適用を書かない。** 全量が来る |
| G-TS-32 | MUST | **Rust 由来のスライスに、置き換え以外の更新関数を作らない。** 「UI から 1 隻だけ直す」という口を生やさない |
| G-TS-33 | MUST | 残り時間・絞り込み結果・並べ替え結果をストアに持たない。描画時に導出する |
| G-TS-34 | MUST | UI 固有の状態（絞り込み条件・選択行・パネル配置）は、Rust 由来のスライスと**別のスライス**に置く。同じオブジェクトに混ぜない |
| G-TS-35 | MUST | `listen` の解除を `useEffect` の cleanup で行う（`listen` は `Promise<UnlistenFn>` を返す） |
| G-TS-36 | MUST | 未観測を「空の配列」で表さない。Rust から来る未観測フラグをそのまま保持する（E-01 と「0 件」は別物） |
| G-TS-37 | MUST | **マスタデータを UI に持ち込まない。** 艦名・装備名の解決を UI 側で行わない（[ADR-0024](../adr/0024-state-sync-granularity.md) 決定 5） |

**良い例**

```ts
// ipc/ — listen の登録はここだけ
events.expeditionUpdated.listen(({ payload }) => {
  useAppStore.getState().replaceExpedition(payload); // 置き換えるだけ
});
```

**悪い例**

```ts
// ❌ UI 側でマージしている
useAppStore.setState((s) => ({
  expedition: { ...s.expedition, rows: mergeRows(s.expedition.rows, payload.rows) },
}));

// ❌ コンポーネントから直接 invoke している
function FleetPanel() {
  useEffect(() => { void invoke("get_fleets"); }, []);
}
```

なぜ悪いのか: マージを書いた瞬間、store は「Rust の状態のコピー」ではなく
「UI が独自に組み立てた状態」になり、**真実の情報源が 2 つになる**
（[ADR-0024](../adr/0024-state-sync-granularity.md) 案 C の却下理由。FUSOU が実際にそうなっている）。
`invoke` が散ると、`bindings.ts` との対応と取りこぼし時の再取得経路が追えなくなる。

### エラー処理と縮退

NFR-003（未知の構造でも停止せず縮退する）の UI 側の担保。
外部仕様は **E-02「1 つのパネルが表示できなくても、他のパネルは表示を続ける」**を要求している
（[fleet-view.md](../spec/external/fleet-view.md) / [timers.md](../spec/external/timers.md)）。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-40 | MUST | **Error Boundary はパネル単位で置く。** アプリ全体を 1 枚で包んで済ませない |
| G-TS-41 | MUST | パネルの外枠（最終更新時刻の表示・E-01 の案内・エラー境界）は **`shared/` の共通部品を使う。** パネルごとに書き直さない（[ADR-0030](../adr/archive/0030-no-named-architecture.md)） |
| G-TS-42 | MUST | 落ちたパネルには「表示できない」と**最後に観測できた時刻**を出す（E-02）。白画面にしない |
| G-TS-43 | MUST | 項目 1 つが欠けただけで `throw` しない。`—` を表示して行は出す（E-04） |
| G-TS-44 | MUST | 並べ替えで `—` の行を末尾に置く。`undefined` を数値として比較しない（E-04） |
| G-TS-45 | MUST | **未知の ID を UI 側で「不明」に変換しない。** E-03 は Rust 側で解決済みの文字列として届く |
| G-TS-46 | SHOULD | Error Boundary は `react-error-boundary` を使う（React 公式が自前実装の代わりに案内している）。→ [未解決事項](#未解決事項) |

**良い例**

```tsx
// shared/ のパネル外枠。13 枚すべてがこれを通る
<Panel title="遠征" observedAt={observedAt} observed={observed}>
  <ExpeditionTable rows={rows} />
</Panel>
```

**悪い例**

```tsx
// ❌ アプリ全体を 1 枚の境界で包んでいる
<ErrorBoundary fallback={<p>エラーが発生しました</p>}>
  <App />
</ErrorBoundary>
```

なぜ悪いのか: **遠征パネル 1 枚のパースが失敗しただけで、入渠も艦隊も見えなくなる。**
外部仕様 E-02 が明示的に禁じている挙動であり、
「ゲームの通信を待たずに手元の情報を見る」という本アプリの目的そのものを壊す。

### アクセシビリティ

[fleet-view.md](../spec/external/fleet-view.md) の
「**色は補助として重ねるが、色を取り除いても状態を区別できる**」は、
艦隊表示だけの都合ではない。**全パネルに効く横断ルールとしてここに昇格させる。**

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-50 | MUST | **状態を色だけで示さない。** 文字・記号・数値のいずれかで判別できるようにし、色はその上に重ねる |
| G-TS-51 | MUST | 数値が得られる項目は、区分名だけでなく**数値を併記する**（fleet-view.md） |
| G-TS-52 | MUST | 押せるものは `button` / `a` で書く。`div` に `onClick` を付けて代用しない |
| G-TS-53 | MUST | アイコンだけのボタンに、読み上げられる名前を与える |
| G-TS-54 | SHOULD | `eslint-plugin-jsx-a11y` の推奨セットを入れ、機械的に検出する。→ [未解決事項](#未解決事項) |

**良い例**

```tsx
// 記号 + 文字 + 数値。色は class で重ねるだけ
<td className={damageClass(ship)}>
  <span aria-hidden="true">{damageMark(ship)}</span> {damageLabel(ship)} {ship.hpNow}/{ship.hpMax}
</td>
```

**悪い例**

```tsx
// ❌ 背景色だけが「大破」を伝えている
<td style={{ background: ship.hpNow / ship.hpMax < 0.25 ? "red" : "transparent" }}>
  {ship.name}
</td>
```

なぜ悪いのか: 色覚特性・輝度の低い画面・スクリーンショットの再圧縮のいずれでも
**情報が消える。** 大破の見落としはそのまま轟沈につながるため、
本アプリでは「見づらい」では済まない。

### ログとプライバシー

[C-04](../spec/constraints.md)（取得したデータを外部に出さない）と
[C-07](../spec/constraints.md)（リポジトリは公開されている前提）の UI 側の担保。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-60 | MUST | **提督名・アカウント ID・サーバ番号・API の生レスポンスを `console.*` に出さない**（C-04 / NFR-011） |
| G-TS-61 | MUST | 例外オブジェクトやペイロードを丸ごとログに流さない。**識別できるのは「どのドメインで失敗したか」までにする** |
| G-TS-62 | MUST | **実測のペイロードをテストの fixture としてコミットしない。** 必要なら値を作り直す（C-07。履歴は消せない） |
| G-TS-63 | SHOULD | 恒久的に残すログは `tauri-plugin-log` 経由にする。`console.*` はデバッグ中だけに使い、コミットしない |

**良い例**

```ts
warn(`遠征パネルの描画に失敗した (observedAt=${observedAt})`);
```

**悪い例**

```ts
console.error("failed", payload);           // ❌ 提督の保有艦一式が出る
console.log("api response", rawResponse);   // ❌ 生のレスポンス
```

なぜ悪いのか: Web Inspector のログはユーザーがそのまま貼って共有しうる。
**開発中に貼られたログが、そのまま公開の場に出る。**
`tauri-plugin-log` を通せばファイルにも残るため、C-04 の範囲がさらに広がる。

### 注入スクリプト

`src/` の他のコードとは規約が違う。**このファイルだけ生 JS である**
（[ADR-0026](../adr/archive/0026-injection-script-build.md)）。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-70 | MUST | **1 ファイル・IIFE・`import` 文なし・npm 依存なし。** 先頭に `// @ts-check` を置き、`allowJs` / `checkJs` と ESLint の対象に含める |
| G-TS-71 | MUST | `bindings.ts` の型は **JSDoc の `import()` 型としてのみ**参照する。値を import しない（cross-origin iframe には `__TAURI_INTERNALS__` が無い） |
| G-TS-72 | MUST | **ページのグローバルを汚さない。** `window.<自前名>` を生やさない。すべてを IIFE のスコープに閉じ、外から見える変更は `XMLHttpRequest` の差し替え 1 点に留める |
| G-TS-73 | MUST | 送り口の分岐（`window.webkit.messageHandlers` / `window.chrome.webview`）を**1 箇所にまとめる**（[ADR-0029](../adr/0029-injection-ipc-transport.md)） |
| G-TS-74 | MUST | **判断を持たない。** `/kcsapi/` の絞り込み以外の解釈・整形・集計をここでしない（[architecture.md](../spec/architecture.md)） |
| G-TS-75 | MUST | **リクエストを発行・改変・再送しない。** `XMLHttpRequest` をサブクラス化し、`loadend` を購読するだけにする（[C-02](../spec/constraints.md)） |

**良い例**

```js
// @ts-check
(() => {
  /** @param {string} json */
  const send = (json) => {
    if (window.webkit?.messageHandlers?.harubridge) {
      window.webkit.messageHandlers.harubridge.postMessage(json);
    } else if (window.chrome?.webview) {
      window.chrome.webview.postMessage(json);
    }
  };
  …
})();
```

**悪い例**

```js
// ❌ グローバルを生やしている（ゲームのページから触れる）
window.harubridgeSend = send;
window.__harubridgeCaptured = [];

// ❌ 判断を持ち込んでいる
if (url.includes("api_port/port")) {
  send(JSON.stringify(summarizeFleets(JSON.parse(xhr.responseText))));
}
```

なぜ悪いのか: グローバルを生やすと、**ゲームのページ側の JS と衝突しうるうえ、
ページから我々の口を叩ける。** [ADR-0029](../adr/0029-injection-ipc-transport.md) が
「受け口は 1 種類に限り、口を広げない」と決めているのと同じ理由である。
解釈を持ち込むと、[architecture.md](../spec/architecture.md) が
パース層に閉じ込めた非公開仕様への依存が、**人間がレビューする 100 行の外側**へ散る。

### テスト

**ドメインのロジックは Rust 側にある。** UI 側で検証する価値があるのは次の 3 つだけである。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-80 | MUST | UI 側のテストは (a) **書式化・並べ替えの純関数**、(b) **ペイロードを受けたストアの置き換え**、(c) **縮退の分岐（E-01 〜 E-06）** の 3 種に限る |
| G-TS-81 | MUST | **制空値・索敵値などのドメイン計算を UI 側でテストしない。** Rust 側にあり、二重に書くと計算式が 2 箇所に生まれる（NFR-009 の出典管理も分裂する） |
| G-TS-82 | MUST | 外部仕様の受け入れ条件に対応するテストには、**対応する ID をテスト名に書く**（`E-04: 欠けた項目の行は — を含めて表示される`） |
| G-TS-83 | MUST | **書式化を純関数として `shared/` に置き、コンポーネントを描画せずに検証できるようにする。** 受け入れ条件の多くはこれで足りる |
| G-TS-84 | SHOULD | 描画を伴うテストは、(c) の縮退の分岐に限って書く。見た目のスナップショットを取らない |
| G-TS-85 | MUST | テストの中で現在時刻を直接読まない。**時刻を引数で渡す**（[ADR-0025](../adr/0025-clock-handling.md) の「絶対時刻 − 現在時刻」をそのまま関数にする） |

外部仕様の受け入れ条件は、**その大半が純関数の表明に落ちる。**
`H:MM:SS`（時は 0 埋めせず、分と秒は 2 桁）/ `—` は並べ替えで末尾 / `不明 (ID: nnnn)` の表示形式は、
いずれもコンポーネントを描画せずに検証できる。

**良い例**

```ts
test("timers.md: 残り時間は H:MM:SS。時は 0 埋めしない", () => {
  expect(formatDuration(3_723_000)).toBe("1:02:03");
  expect(formatDuration(59_000)).toBe("0:00:59");
});

test("E-04: — の行は並べ替えで末尾に来る", () => {
  expect(sortByHp([{ hp: undefined }, { hp: 10 }]).at(-1)?.hp).toBeUndefined();
});
```

**悪い例**

```ts
// ❌ Rust 側にある計算を UI で再実装してテストしている
test("制空値", () => expect(fighterPower(gears)).toBe(312));

// ❌ 時刻を関数の内側で読んでいる。テストのたびに結果が変わる
export function remaining(completesAt: number) {
  return completesAt - Date.now();
}
```

なぜ悪いのか: 前者は [ADR-0024](../adr/0024-state-sync-granularity.md) 決定 5
（計算は Rust 側で済ませる）に反し、**計算式の出典（NFR-009）が 2 箇所に分裂する。**
後者は関数が現在時刻に依存するため、テストで時刻を固定するしかなくなる。
引数で受け取れば、テストは単なる算術になる。

### ESLint / Prettier

[ADR-0019](../adr/0019-linter.md) が `eslint` + `prettier` を、
**型情報つきのリント**を有効にする前提で採用している。

| ID | 強度 | ルール |
| --- | --- | --- |
| G-TS-90 | MUST | **推奨セットで検出できるものを、このガイドラインに書き写さない。** ルールの重複は必ず片方が腐る |
| G-TS-91 | MUST | `typescript-eslint` の**型情報を要求する推奨セット**（`recommended-type-checked` 以上）を土台にする。[ADR-0019](../adr/0019-linter.md) はこの検出力のために `biome` を捨てた |
| G-TS-92 | MUST | `eslint-plugin-react-hooks` の推奨セットを入れる（Rules of Hooks の機械検出） |
| G-TS-93 | MUST | **`import/no-restricted-paths` の zone 定義を消さない・緩めない。** これが [ADR-0030](../adr/archive/0030-no-named-architecture.md) の 3 本目の境界（`shared → features → app` の一方向）を守る唯一の手段である |
| G-TS-94 | MUST | `eslint-disable` は**行単位**（`eslint-disable-next-line`）・**ルール名指定**・**理由コメント**の 3 点をそろえる。ファイル先頭での一括無効化をしない |
| G-TS-95 | MUST | **`import/no-restricted-paths` と型安全性に関わるルール（`no-explicit-any`、`no-floating-promises`、`no-unsafe-*`）を `eslint-disable` しない。** ここを黙らせるくらいなら設計を変える |
| G-TS-96 | MUST | Prettier の設定を増やさない。既定のまま使い、整形について議論しない |
| G-TS-97 | MUST | `pnpm lint` が通らない状態でコミットしない（`main` に直接コミットするため。[ADR-0014](../adr/0014-trunk-based-on-main.md)） |

**良い例**

```ts
// eslint-disable-next-line @typescript-eslint/no-unnecessary-condition -- 注入スクリプト由来で、型はあるが実行時には欠けうる
if (payload.observedAt == null) return;
```

**悪い例**

```ts
/* eslint-disable */              // ❌ ファイルごと黙らせている
// eslint-disable-next-line       // ❌ ルール名も理由も無い
```

なぜ悪いのか: ルール名を書かない `eslint-disable` は、
**そこで無効になっている検査が何かを誰も追えない。**
ファイル単位の無効化は、後から追加された行にも無条件で効き続ける。
[ADR-0019](../adr/0019-linter.md) の決め手は「検出漏れは毎日効く」であり、
自分で検出漏れを作る行為はその判断と正面から矛盾する。

## 具体例

各ルールの節に良い例と悪い例を置いた。ここでは**節をまたぐ 1 枚のパネル**を通しで示す。

**良い例 —— 1 枚のパネル**

```tsx
// features/expedition/ExpeditionPanel.tsx
export function ExpeditionPanel() {
  // 必要な最小単位だけ購読する（G-TS-26 / 27）
  const state = useAppStore(useShallow((s) => s.expedition));

  return (
    // 外枠は shared/。E-01 の案内・最終更新時刻・エラー境界を含む（G-TS-40 / 41）
    <Panel title="遠征" observedAt={state.observedAt} observed={state.observed}>
      <table>
        <tbody>
          {state.rows.map((row) => (
            <ExpeditionRow key={row.fleetId} row={row} />   // 安定した key（G-TS-28）
          ))}
        </tbody>
      </table>
    </Panel>
  );
}

function ExpeditionRow({ row }: { row: ExpeditionRow }) {
  return (
    <tr>
      <td>{row.fleetName}</td>
      {/* 得られない項目は — を出し、行は表示する（G-TS-43） */}
      <td>{row.missionName ?? "—"}</td>
      {/* tick を購読するのはこのセルの中だけ（G-TS-25） */}
      <td><Countdown completesAt={row.completesAt} /></td>
    </tr>
  );
}
```

このパネルには次が **1 つも無い**。それがこのガイドラインの要点である。

- `invoke` / `listen`（`ipc/` にある。G-TS-30）
- `setInterval`（アプリ全体で 1 本。G-TS-24）
- マージ処理・`setState` による書き戻し（G-TS-31 / 32）
- マスタ参照・艦名の解決・制空値の計算（Rust 側。G-TS-37 / 81）
- `try` / `catch` によるパネルの握りつぶし（エラー境界が受ける。G-TS-40）

## 例外

- **クラスコンポーネントの禁止（G-TS-20）の例外は Error Boundary だけ。**
  React 公式が「関数コンポーネントでは書けない」と明記しているためである。
  `react-error-boundary` を使う場合、自前のクラスは 1 つも要らない
- **`unknown` を経由してよいのは、注入スクリプトから届いた値と `JSON.parse` の結果だけ**（G-TS-01）。
  `bindings.ts` を通る値には型が付いている
- **`eslint-disable` を書いてよい条件**（G-TS-94）: 行単位・ルール名指定・理由コメントの 3 点がそろい、
  かつ G-TS-95 のルール群でないこと。
  この 3 点が書けないなら、それは**設計のほうを直す合図**である
- 上記以外に例外を認めない。**特に G-TS-31（マージを書かない）と G-TS-40（境界はパネル単位）は
  例外なし。** どちらも破った時点で ADR-0024 / 外部仕様 E-02 が成立しなくなる

## 未解決事項

- `TODO(未確定)`: **`react-error-boundary` の採否**（G-TS-46）。
  React 公式が案内しているが、[ADR-0018](../adr/0018-dependencies.md) の依存一覧に無い。
  **依存を 1 つ増やす判断であり、ADR-0018 系列の追記が要る**
- `TODO(未確定)`: **`eslint-plugin-jsx-a11y` の採否**（G-TS-54）。同上。
  加えて `eslint-plugin-import`（`import/no-restricted-paths`）は
  [ADR-0030](../adr/archive/0030-no-named-architecture.md) の「影響」で必要と明記されているが、
  こちらも ADR-0018 の一覧には未記載である
- `TODO(要検証)`: `typescript-eslint` の推奨セットを
  `recommended-type-checked` に留めるか `strict-type-checked` まで上げるか。
  後者は semver の対象外（マイナー更新で新ルールが入りうる）と公式が明記しており、
  CI が突然落ちる面が広がる。**実際に走らせて指摘の量を見てから決める**
- `TODO(未確定)`: ファイル名の表記（G-TS-15）。
  `PascalCase.tsx` / `camelCase.ts` の併用ではなく全体を `kebab-case` に寄せる流儀もある
  （bulletproof-react）。**React 公式に規範が無い**ため
  （[ADR-0030](../adr/archive/0030-no-named-architecture.md) 根拠表）、どちらでも通る。実装着手時に 1 つ選ぶ
- `TODO(未確定)`: UI コンポーネントライブラリ未選定
  （[ADR-0018](../adr/0018-dependencies.md) の未解決事項）。
  選定後、G-TS-52 / 53（意味のある要素・アクセシブルな名前）をどこまでライブラリに任せるかを見直す
- `TODO(未確定)`: E2E テストの有無（[ADR-0018](../adr/0018-dependencies.md) の未解決事項）。
  本ガイドラインの「テスト」節は単体テストの範囲しか定めていない
