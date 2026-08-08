//! Harubridge のコア。通信データの解釈・状態保持・永続化・任務の計数を担う。
//!
//! このクレートは `tauri` に依存しない（ADR-0032 §1）。
//! IPC・ウィンドウ・プラグインの都合は `src-tauri` 側に置くこと。
//! module の分割方針は ADR-0032 §5（関心事ごと。層の名前を持ち込まない）。

/// コアが殻（`src-tauri`）から結線されていることを確認するための暫定関数。
/// 最初の実機能が入った時点で削除してよい。
pub fn core_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_not_empty() {
        assert!(!super::core_version().is_empty());
    }
}
