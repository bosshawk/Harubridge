//! 追跡対象ファイルを走査し、観測データの混入を検出したら異常終了する。
//!
//! 走査対象を `git ls-files` から取るのは、**コミットされうるものだけを見るため**である。
//! 作業ツリーに置いただけのファイル（`.local/` など）は git が返さないので対象にならない。

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use repo_guard::inspect;

fn main() -> ExitCode {
    let root = match repo_root() {
        Ok(root) => root,
        Err(message) => {
            eprintln!("リポジトリ根を特定できない: {message}");
            return ExitCode::FAILURE;
        }
    };

    let files = match tracked_files(&root) {
        Ok(files) => files,
        Err(message) => {
            eprintln!("追跡対象ファイルを取得できない: {message}");
            return ExitCode::FAILURE;
        }
    };

    let mut violations = Vec::new();
    for relative in &files {
        // 削除済みだがインデックスに残っているものは読めない。走査の対象にならないだけで問題ない
        let Ok(bytes) = std::fs::read(root.join(relative)) else {
            continue;
        };
        if let Some(violation) = inspect(relative, &bytes) {
            violations.push((relative.clone(), violation));
        }
    }

    if violations.is_empty() {
        println!("観測データの混入なし（{} ファイルを検査）", files.len());
        return ExitCode::SUCCESS;
    }

    eprintln!("観測データが混入している可能性がある:");
    for (path, violation) in &violations {
        eprintln!("  {path}: {violation}");
    }
    eprintln!();
    eprintln!("生の通信データはリポジトリに入れない。テストの入力は生成器の出力だけを使う。");
    ExitCode::FAILURE
}

fn repo_root() -> Result<PathBuf, String> {
    let output = run_git(Path::new("."), &["rev-parse", "--show-toplevel"])?;
    let text = String::from_utf8(output).map_err(|e| e.to_string())?;
    Ok(PathBuf::from(text.trim()))
}

fn tracked_files(root: &Path) -> Result<Vec<String>, String> {
    // パス区切りとして安全なのは NUL だけ。改行を含むファイル名でも壊れない
    let output = run_git(root, &["ls-files", "-z"])?;
    let text = String::from_utf8(output).map_err(|e| e.to_string())?;
    Ok(text
        .split('\0')
        .filter(|entry| !entry.is_empty())
        .map(str::to_owned)
        .collect())
}

fn run_git(directory: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let output = Command::new("git")
        .current_dir(directory)
        .args(args)
        .output()
        .map_err(|e| format!("git を起動できない: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(output.stdout)
}
