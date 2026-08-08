# ADR-0026: 注入スクリプトは生 JS 1 ファイルのまま持ち、型検査だけを TypeScript に任せる

- ステータス: Superseded by [ADR-0032](0032-repository-structure.md)（決定は変更なしで統合された）
- 日付: 2026-08-03
- 決定者: 承認待ち（`Accepted` にする時点で承認者名に書き換える）
- 関連: [ADR-0016](0016-tech-stack.md)（観測方式と「型で守る」方針）,
  [ADR-0018](0018-dependencies.md)（`tauri-specta` による型の受け渡し）,
  [ADR-0019](0019-linter.md)（ESLint + Prettier / 型情報つきリント）,
  [architecture.md](../spec/architecture.md), [C-02 / C-04](../spec/constraints.md)

## 背景と課題

[ADR-0016](0016-tech-stack.md) の観測方式は**ページ内での XHR フック**である。
[architecture.md](../spec/architecture.md) はこのスクリプトに次の役割を与えている。

- ゲームのフレーム内で XHR を観測し、Rust へ渡す。**ページ内でのみ動き、判断を持たない**
- **絞り込みは注入スクリプトの中で行う。`/kcsapi/` 以外は Rust コアへ渡さない**
  （実測 2026-08-02: `/kcsapi/` 18 件に対しそれ以外が 216 件。**境界はページ内に置く**）
- `XMLHttpRequest` を**サブクラス化し、`loadend` を購読するだけ**とする（[C-02](../spec/constraints.md)）

同等のことを poi は**生 JS 1 ファイル**で行っている。
一方 [ADR-0016](0016-tech-stack.md) は「型の強い言語のほうがメンテナンス性が高い」
「人間による常時レビューが無い体制では、コンパイラが最後の防波堤になる」を決め手にしており、
[ADR-0019](0019-linter.md) はその一貫性を理由にリンタを選び直している。

**この 100 行を TypeScript でビルドするのか、生 JS のまま持つのかを決める必要がある。**
「小さいから生 JS でよい」で済ませると、上記 2 本の決定と正面から食い違う。

### 確認した事実（一次ソース。2026-08-03 取得）

| # | 事実 | 出典 |
| --- | --- | --- |
| 1 | `WebviewBuilder::initialization_script_for_all_frames(script: impl Into<String>)`。**受け取るのは文字列であり、ファイルパスではない** | tauri `crates/tauri/src/webview/mod.rs`（dev） |
| 2 | Windows は常にサブフレームへも注入する、と Tauri のドキュメントが明記 | 同上 / [docs.rs](https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html) |
| 3 | wry は `with_initialization_script_for_main_only(js, for_main_only)` を持ち、macOS では `WKUserScript(..., AtDocumentStart, for_main_only)` にそのまま渡す | wry `src/lib.rs` / `src/wkwebview/mod.rs`（dev） |
| 4 | **Tauri 自身のブートストラップはすべて `for_main_frame_only: true`。** `isTauri` / `__TAURI_INTERNALS__` / invoke 初期化 / metadata / プラグインの全スクリプトが `main_frame_script()` で積まれる | tauri `crates/tauri/src/manager/webview.rs`（dev） |
| 5 | `@tauri-apps/api` の `invoke()` は `window.__TAURI_INTERNALS__.invoke(cmd, args, options)` を呼ぶだけ | tauri `packages/api/src/core.ts`（dev） |
| 6 | `tauri-specta` の生成物は先頭で `import { invoke as __TAURI_INVOKE, Channel } from "@tauri-apps/api/core"` を**値として** import する。同じファイルに `export type ...` の宣言も並ぶ | specta-rs/tauri-specta `examples/app/src/bindings.ts` |
| 7 | wry の `window.ipc` も macOS では `for_main_only = true` で注入される | wry `src/wkwebview/mod.rs`（dev） |
| 8 | WebView2 の `AddScriptToExecuteOnDocumentCreated` は全フレームに効くが、`WebMessageReceived` は **"the top-level document" が `postMessage` したときに発火する** | [Microsoft Learn ICoreWebView2](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2) |
| 9 | poi の `assets/js/xhr-hack.js` は **101 行の生 JS**（`module.exports`）。冒頭コメントに「`contextBridge.executeInMainWorld` でページの main world へ**シリアライズされる**。自己完結を保つこと。参照してよいのはグローバルと `window.poiPreloadBridge` だけ」とある | poooi/poi `assets/js/xhr-hack.js`（master） |
| 10 | poi 本体は TypeScript + Babel + Gulp（`typecheck: tsc --noEmit`, `@babel/preset-typescript`）。**TS を使える構成でありながら、この 1 ファイルだけ生 JS である** | poooi/poi `package.json`（master） |
| 11 | ES5 へ downlevel すると `Error` / `Array` / `Map` などの**組み込みの継承が壊れる**（`new.target` を再現できないため） | [TypeScript Breaking Changes（2.1）](https://github.com/microsoft/TypeScript/wiki/Breaking-Changes) |
| 12 | esbuild は構文の downlevel 時に**ヘルパ関数を出力へ挿入する**（`a ** b` → `__pow(a, b)`）。かつ「JavaScript のスコープ規則が改変されていないことを前提にする」と明記 | [esbuild Content Types](https://esbuild.github.io/content-types/) |
| 13 | JSDoc の `@import` / `import()` 型は「**実行時には何も import せず、持ち込まれる名前は JSDoc の中でしか使えない**」 | [TypeScript JSDoc Reference](https://www.typescriptlang.org/docs/handbook/jsdoc-supported-types.html) |
| 14 | Vite の `build.lib.formats` は `'es' \| 'cjs' \| 'umd' \| 'iife'` を取る。`beforeBuildCommand` は「`tauri build` の前に走らせるシェルコマンド」 | [Vite build options](https://vite.dev/config/build-options.html) / [Tauri config](https://v2.tauri.app/reference/config/) |
| 15 | ブラウザ拡張のフレームワーク WXT は、main world で動かすスクリプトを**別エントリポイント**として扱い、拡張 API に触れない前提で書かせる | [WXT Content Scripts](https://wxt.dev/guide/essentials/content-scripts.html) |

### 事実 4〜6 の帰結（本 ADR の前提として重要）

**`tauri-specta` が生成する `commands` / `events` は、注入スクリプトからは動かない。**
ゲームは `osapi.dmm.com` 配下の cross-origin iframe で動いており（実測 2026-08-02）、
そのフレームには `window.__TAURI_INTERNALS__` が存在しないためである（事実 4 + 5）。
UI（メインフレーム）と注入スクリプトでは **IPC の経路がそもそも違う。**

一方、生成物には Rust の型に対応する `export type ...` が並んでいる（事実 6）。
**型宣言だけなら共有できる。** `import type` および JSDoc の `import()` 型は
実行時に何も残らない（事実 13）ため、注入スクリプトに依存を持ち込まない。

→ **「値は共有できない。型だけ共有できる」**というのが本 ADR の出発点である。

## 決定

**注入スクリプトはリポジトリ内に生 JS 1 ファイルとして置き、バンドラを通さずそのまま
`initialization_script_for_all_frames` に渡す。**

ただし「型を当てない生 JS」にはしない。次を必須とする。

- ファイル先頭に `// @ts-check` を置き、`tsconfig` の `allowJs` / `checkJs` の対象に含める。
  **DOM の型（`lib: ["DOM"]`）を当てて TypeScript コンパイラの検査を通す**
- [ADR-0019](0019-linter.md) の ESLint（型情報つき）の対象に含める
- Rust とのメッセージ境界の型は、`tauri-specta` の生成物から
  **JSDoc の `@import` / `import()` 型としてのみ参照する**（実行時の依存はゼロ）
- **import 文を書かない。npm パッケージに依存しない。IIFE として自己完結させる**

## 検討した選択肢

### 案 A: 生 JS 1 ファイル + `@ts-check` + JSDoc（採用）

- 概要: 注入するテキストそのものをリポジトリに置き、Rust から埋め込む。
  型検査とリントは TypeScript / ESLint に任せ、**トランスパイルもバンドルもしない**
- 利点:
  - **リポジトリのテキストと、ページに注入されるテキストが同一である。**
    [C-02](../spec/constraints.md)（通信は読み取り専用）を人間が目視で確認する対象と、
    実際にゲームのページで走るものが一致する
  - ヘルパ関数も polyfill も混入しない（事実 12 の懸念が構造的に発生しない）。
    組み込みのサブクラス化が downlevel で壊れる経路も無い（事実 11）
  - 生成物が無いため、`cargo build` / `cargo test` / rust-analyzer が
    フロントエンドのビルド順に依存しない
  - Web Inspector に出るコードがそのまま原文であり、**ソースマップを必要としない**
  - 型検査の強さは案 B とほぼ同じ。`checkJs` は同じコンパイラが同じ `lib` で検査する
- 欠点:
  - 型注釈が JSDoc になり、TypeScript の構文より冗長
  - フロントエンドが `.ts`、この 1 ファイルだけ `.js` という**不揃い**が残る
  - ファイルが育って分割やライブラリ利用が必要になった時点で、この決定は成立しなくなる

### 案 B: TypeScript 1 ファイルを `tsc` でトランスパイルする（バンドルはしない）

- 概要: `.ts` で書き、`target: ESNext` の `tsc` で型を落とした `.js` を出し、それを埋め込む
- 利点: 記述が素直な TypeScript になり、フロントエンドと拡張子が揃う
- 欠点: 出力が生成物になる。Rust が埋め込む対象が「ビルドしないと存在しないファイル」になり、
  `cargo` 単体でのビルドとフロントエンドのビルドに順序制約が生まれる（事実 14 の
  `beforeBuildCommand` は `tauri build` の前段には効くが、`cargo test` には効かない）
- 却下理由: **型検査の強さは案 A と変わらないのに、生成物と順序制約だけが増える**
  （import を書かない前提では、`tsc` の仕事は型注釈の除去だけになる）

### 案 C: TypeScript + バンドラ（Vite の `iife` 出力 / esbuild）で 1 ファイルに束ねる

- 概要: 注入スクリプトを 2 つめのビルドエントリとし、`build.lib.formats: ['iife']` で
  単一ファイルを出す（事実 14）。ブラウザ拡張の main world スクリプトで一般的な構成（事実 15）
- 利点:
  - ファイルを分割でき、npm のライブラリも使える
  - `import type` を素直に書ける
- 欠点:
  - **注入されるのが生成物になる。** ヘルパ関数の混入は `target` 次第で発生し（事実 12）、
    esbuild は「スコープ規則が改変されていない」ことを前提に置く。
    注入先はゲームのページであり、こちらが前提を保証できる場所ではない
  - 案 B の順序制約に加えて、バンドラ設定（format / target / minify / sourcemap）が
    **監査対象の増分**になる
- 却下理由: **利点（分割と npm 依存）が、いま満たしてはいけない要件である。**
  architecture.md は注入スクリプトを「ページ内でのみ動き、判断を持たない」ものと定めており、
  分割が要るほど育てないことが前提。poi も TS を使える構成でこの 1 ファイルだけ
  生 JS に留めている（事実 9 / 10）

### 案 D: 型検査もしない素の生 JS（poi と同じ形）

- 概要: poi の `xhr-hack.js` と同じく、型に関する仕組みを一切入れない
- 却下理由: [ADR-0016](0016-tech-stack.md) と [ADR-0019](0019-linter.md) が
  「コンパイラとリンタが最後の防波堤」を根拠に選定を行っており、**この 1 ファイルだけ
  例外にする理由が無い**（`@ts-check` の追加コストは 1 行である）。

## 決め手

**「人間がレビューするテキストと、ゲームのページで実行されるテキストが同一であること」を取り、
`.ts` という拡張子とモジュール分割の自由を捨てた。**

[C-02](../spec/constraints.md) を担保しているのはこの 100 行だけであり、
そこが生成物になると、[ADR-0003](0003-agent-driven-development.md) の体制で
人間が確認できるものが「入力」だけになる。

## 影響

- 実装への影響:
  - 注入スクリプトは **1 ファイル・import なし・npm 依存なし・IIFE** で書く
  - `tsconfig` に `allowJs` / `checkJs` と `lib: ["DOM", ...]` の設定が要る。
    ESLint（[ADR-0019](0019-linter.md)）の対象にもこのファイルを含める
  - `tauri-specta` の生成物は **UI 側だけが値として使う。**
    注入スクリプトからは JSDoc の型としてのみ参照する
- ドキュメントへの影響: [architecture.md](../spec/architecture.md) の構造は変わらない
  （境界の位置も責務も同じ）ため、更新は不要
- 取り消す場合のコスト: **低。** 100 行を `.ts` に移してビルドエントリを 1 つ足すだけで案 C に移れる。
  逆方向も同じ
- **本 ADR を見直す条件**: 「1 ファイル・import なし・npm 依存なし」のいずれかが崩れたとき。
  そのときは案 C を再検討する

## 未解決事項

- `TODO(未確定)`: **注入スクリプトから Rust へメッセージを渡す経路。**
  今回の調査で、cross-origin iframe には `__TAURI_INTERNALS__` も `window.ipc` も存在せず
  （事実 4 / 7）、Windows では `WebMessageReceived` が top-level document のみで発火する
  （事実 8）ことが分かった。**本 ADR の判断はこの経路に依存しないが、別途決める必要がある。**
  経路が決まるまで、共有できる型が実際に生まれるかも確定しない
- `TODO(要検証)`: `WKUserScript` で注入したコードに `//# sourceURL` / ソースマップが効くか。
  一次ソースを見つけられなかった。案 A では原文がそのまま注入されるため当面問題にならないが、
  案 C へ移る場合はここが前提条件になる
- `TODO(要検証)`: `checkJs` + JSDoc で、`XMLHttpRequest` のサブクラス化に対して
  TypeScript がどこまで有効な検査を行えるか。実コードで確認していない
- `TODO(要検証)`: `fetch` / WebSocket は現時点で未使用（実測 2026-08-02）。
  使われ始めた場合、注入スクリプトの規模が変わり、本 ADR の見直し条件に触れうる
