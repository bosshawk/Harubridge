"use strict";
// 配信元ページの main world で動く表示調整スクリプト。
//
// 規律:
// - 値の import を書かない。npm に依存しない。IIFE として自己完結させる
//   （型だけは `import type` で参照してよい。コンパイル時に消える）
// - **`<style>` を 1 枚足す以外のことをしない。** 既存の要素を消したり動かしたり、
//   ページの関数を差し替えたり、通信に触れたりしない（architecture.md の 1 段目）
// - 観測（kcsapi-hook.ts）と混ぜない。役割ごとにファイルを分ける
//
// 生成物 page-style.js は `pnpm build:injected` で作り、コミットする。
// 手で編集しないこと。
(() => {
    "use strict";
    const STYLE_ID = "harubridge-page-style";
    // ゲームを載せている要素の id。観測: docs/kancolle/api/overview.md
    const GAME_FRAME_ID = "game_frame";
    // 既存の要素を消す代わりに、画面いっぱいの背景で覆い、
    // その上にゲーム画面だけを載せる。
    // **配信元ページの構造を列挙しない**ので、DMM 側で見出しや広告が増減しても破綻しない。
    // 列挙する方式（消したい要素を id や class で指定する）は、
    // 増えた要素を消し漏らすと画面に出てしまう。こちらは既定が「隠れる」側になる。
    const CSS = `
html, body {
  margin: 0 !important;
  padding: 0 !important;
  overflow: hidden !important;
}
/* 画面全体を覆う背景。ゲーム画面より 1 つ下に敷く */
html::before {
  content: "" !important;
  position: fixed !important;
  inset: 0 !important;
  background: #000 !important;
  z-index: 2147483646 !important;
}
#${GAME_FRAME_ID} {
  position: fixed !important;
  top: 0 !important;
  left: 0 !important;
  z-index: 2147483647 !important;
  border: 0 !important;
  margin: 0 !important;
}
`;
    function apply() {
        if (document.getElementById(STYLE_ID) !== null) {
            return true;
        }
        // ゲームを載せる要素が無いページ（ログイン画面・配信元のお知らせ等）では
        // 何もしない。素のページがそのまま表示される
        if (document.getElementById(GAME_FRAME_ID) === null) {
            return false;
        }
        const head = document.head;
        if (head === null) {
            return false;
        }
        const style = document.createElement("style");
        style.id = STYLE_ID;
        style.textContent = CSS;
        head.appendChild(style);
        return true;
    }
    if (apply()) {
        return;
    }
    // ゲームを載せる要素はページ読み込みの後から挿入される。出現するまで待つ
    const observer = new MutationObserver(() => {
        if (apply()) {
            observer.disconnect();
        }
    });
    observer.observe(document.documentElement, {
        childList: true,
        subtree: true,
    });
})();
