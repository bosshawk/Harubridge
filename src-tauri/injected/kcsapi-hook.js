"use strict";
// ゲームページの main world で動く観測スクリプト。
//
// 規律:
// - 値の import を書かない。npm に依存しない。IIFE として自己完結させる
//   （型だけは `import type` で参照してよい。コンパイル時に消える）
// - 通信への介入（停止・改変・再送）を行わない。読み取りのみ（C-02）
// - 判断を持たない。/kcsapi/ の絞り込みと転送だけを行う（architecture.md）
//
// 生成物 kcsapi-hook.js は `pnpm build:injected` で作り、コミットする。
// 手で編集しないこと。
(() => {
    "use strict";
    // TODO(実装): XMLHttpRequest のサブクラス化による /kcsapi/ 応答の観測。
    // Rust への転送経路は未決。決まり次第ここに実装する。
})();
