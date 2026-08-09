//! Harubridge のコア。通信データの解釈・状態保持・永続化・任務の計数を担う。
//!
//! このクレートは `tauri` に依存しない。
//! IPC・ウィンドウ・プラグインの都合は `src-tauri` 側に置くこと。
//! module は関心事ごとに分割し、層の名前（domain / usecase など）を持ち込まない。

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
