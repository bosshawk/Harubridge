# 暫定メモ（廃止予定）

> **このディレクトリは廃止された**（[ADR-0009](../adr/0009-notes-as-github-issues.md)）。
> **新しいメモをここに追加しない。**

その場の方針・検討メモ・調査ログ・保留中の論点は **GitHub Issue** に置く。

## なぜ移したか

捨てる前提の資料が git 履歴に永久に残り、棚卸しが手作業になっていた。
Issue には open / closed の状態があり、閉じれば視界から消えて履歴には残る。
詳細は [ADR-0009](../adr/0009-notes-as-github-issues.md)。

## いま何をすべきか

| したいこと | どうするか |
| --- | --- |
| 論点・調査結果を書き留める | `gh issue create` で Issue を立てる |
| 決着した | ADR を起票し、Issue に ADR へのリンクを貼って閉じる |
| 過去の検討を探す | `gh issue list --state all` |

エージェント向けの手順は `.claude/skills/capture-issue/SKILL.md`。

## 移行状況

GitHub リポジトリが未作成のため、以下のメモが未移行。
**リポジトリ作成時に Issue 化し、このディレクトリごと削除する。**

| ファイル | 移行先 |
| --- | --- |
| [2026-08-01-tech-stack-open-questions.md](2026-08-01-tech-stack-open-questions.md) | `TODO`: Issue 化する |
