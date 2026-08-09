//! 追跡対象ファイルに、艦これから観測した通信データが混入していないことを検査する。
//!
//! # なぜフィールド名で検査しないか
//!
//! 艦これの API は公開されておらず、**どのフィールドが個人を指すのかを列挙しきれない。**
//! 「識別子を並べて、それが無いことを確かめる」方式は、列挙の漏れが無いことを原理的に
//! 証明できない。実在の専用ブラウザでも、除去対象の正規表現から自由記述欄が漏れている例がある。
//!
//! そこで検査するものを、**証明できない性質から機械的に判定できる性質へ置き換える。**
//! 「個人情報が入っていないこと」ではなく「観測データの形をしていないこと」を見る。
//! 形は 2 つしかない —— `svdata=` で始まること、`api_result` と `api_data` を同時に持つこと。
//! どちらもフィールドの意味を知らずに判定できる。
//!
//! # なぜ CI で回すか
//!
//! リポジトリは公開されており、**いちど混入すれば履歴から消せない。**
//! `.gitignore` は `git add -f` で、コミットフックは無効化で越えられるため、
//! どちらも保証にならない。この検査は越えられない場所で回す必要がある。

use std::path::Path;

/// 検査から除外するパス（リポジトリ根からの相対）。
///
/// テストフィクスチャは合成データに限り、生成器（Rust コード）の出力とバイト一致させる。
/// **生成器の出力だけがここに載る。**
///
/// 現時点では生成器が存在しないため空である。空であることに意味がある ——
/// 手で置いた実測データはここに載りようがなく、必ず検出される。
pub const GENERATED_FIXTURES: &[&str] = &[];

/// 検出した混入の種類。
#[derive(Debug, PartialEq, Eq)]
pub enum Violation {
    /// ファイルが `svdata=` で始まる。艦これのレスポンスはこの形で返る。
    SvdataPrefix,
    /// JSON が `api_result` と `api_data` を同時に持つ。`/kcsapi/` のレスポンスの外枠。
    KcsapiEnvelope,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SvdataPrefix => write!(f, "`svdata=` で始まる（艦これのレスポンスの形）"),
            Self::KcsapiEnvelope => write!(
                f,
                "`api_result` と `api_data` を同時に持つ JSON（/kcsapi/ のレスポンスの形）"
            ),
        }
    }
}

/// 1 ファイルを検査する。`path` はリポジトリ根からの相対パス。
///
/// 判定できないもの（生成器の出力・UTF-8 でないもの・壊れた JSON）は `None` を返す。
/// **見逃しではなく対象外である。** 実測データはいずれの経路でも `None` にならない。
pub fn inspect(path: &str, bytes: &[u8]) -> Option<Violation> {
    if is_generated_fixture(path) {
        return None;
    }

    if bytes.starts_with(b"svdata=") {
        return Some(Violation::SvdataPrefix);
    }

    if has_json_extension(path) {
        let text = std::str::from_utf8(bytes).ok()?;
        let value: serde_json::Value = serde_json::from_str(text).ok()?;
        if contains_kcsapi_envelope(&value) {
            return Some(Violation::KcsapiEnvelope);
        }
    }

    None
}

/// 生成器の出力として登録済みか。
pub fn is_generated_fixture(path: &str) -> bool {
    GENERATED_FIXTURES.contains(&path)
}

fn has_json_extension(path: &str) -> bool {
    Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

/// 文書のどこかに `api_result` と `api_data` を同時に持つオブジェクトがあるか。
///
/// 最上位だけを見ないのは、レスポンスを何かで包んで保存した形も同じ危険を持つため。
fn contains_kcsapi_envelope(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if map.contains_key("api_result") && map.contains_key("api_data") {
                return true;
            }
            map.values().any(contains_kcsapi_envelope)
        }
        serde_json::Value::Array(items) => items.iter().any(contains_kcsapi_envelope),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svdata_prefix_is_detected_regardless_of_extension() {
        let body = br#"svdata={"api_result":1}"#;
        assert_eq!(inspect("captured.txt", body), Some(Violation::SvdataPrefix));
        assert_eq!(
            inspect("captured.json", body),
            Some(Violation::SvdataPrefix)
        );
    }

    #[test]
    fn svdata_must_be_at_the_head() {
        // 説明文の中に語が出てくるだけのものは検出しない。誤検出はこの検査を無視させる
        let body = "レスポンスは svdata= で始まる".as_bytes();
        assert_eq!(inspect("docs/note.md", body), None);
    }

    #[test]
    fn kcsapi_envelope_is_detected() {
        let body = br#"{"api_result":1,"api_data":{"api_member_id":"12345"}}"#;
        assert_eq!(
            inspect("fixture.json", body),
            Some(Violation::KcsapiEnvelope)
        );
    }

    #[test]
    fn kcsapi_envelope_is_detected_when_wrapped() {
        // 包んで保存しても同じ危険を持つため、入れ子も見る
        let body = br#"{"captured":[{"response":{"api_result":1,"api_data":{}}}]}"#;
        assert_eq!(inspect("log.json", body), Some(Violation::KcsapiEnvelope));
    }

    #[test]
    fn one_of_the_two_keys_is_not_enough() {
        let body = br#"{"api_result":1}"#;
        assert_eq!(inspect("partial.json", body), None);
    }

    #[test]
    fn json_check_applies_to_json_files_only() {
        // 文書に形を説明として書けるようにする。書けないと、この規律自体を文書化できない
        let body = br#"{"api_result":1,"api_data":{}}"#;
        assert_eq!(inspect("docs/kancolle/api/overview.md", body), None);
    }

    #[test]
    fn ordinary_repository_files_pass() {
        let quests = br#"{"version":1,"quests":[{"id":337,"max":3,"period":"daily"}]}"#;
        assert_eq!(inspect("data/kancolle/quests.json", quests), None);
        assert_eq!(inspect("src/main.tsx", b"export {}"), None);
    }

    #[test]
    fn broken_or_binary_content_is_out_of_scope() {
        assert_eq!(inspect("broken.json", b"{ not json"), None);
        assert_eq!(inspect("icon.json", &[0xff, 0xfe, 0x00]), None);
    }

    #[test]
    fn nothing_is_excluded_yet() {
        // 生成器が存在しない間は除外がゼロであること。
        // ここが埋まるのは生成器を作るときだけで、それ以外の追加は混入の抜け道になる
        assert!(GENERATED_FIXTURES.is_empty());
        assert!(!is_generated_fixture(
            "crates/harubridge-core/tests/fixtures/port.json"
        ));
    }
}
