// Rust との通信（invoke / listen）はこのディレクトリの外に書かない。
// ESLint の no-restricted-imports が @tauri-apps/api の直接 import を外側で禁止している。
// tauri-specta の配線（bindings.ts の生成）が入った時点で、listen 登録と初期 pull をここに実装する。
export {};
