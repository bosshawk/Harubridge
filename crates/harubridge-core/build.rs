//! `data/kancolle/*.json` をビルド時に検証する。
//! 不正な JSON があればビルドが落ちる。
//! 現時点では検証のみ。データを使う実装が入った時点で `OUT_DIR` への生成を足す。

use std::fs;
use std::path::Path;

fn main() {
    // data/ はパッケージの外にあるため、明示しないと Cargo は変更を検出しない
    println!("cargo::rerun-if-changed=../../data/kancolle");

    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/kancolle");
    for entry in fs::read_dir(&dir).expect("data/kancolle を読めない") {
        let path = entry.expect("data/kancolle のエントリを読めない").path();
        if path.extension().is_some_and(|e| e == "json") {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} を読めない: {e}", path.display()));
            serde_json::from_str::<serde_json::Value>(&text)
                .unwrap_or_else(|e| panic!("{} が不正な JSON: {e}", path.display()));
        }
    }
}
