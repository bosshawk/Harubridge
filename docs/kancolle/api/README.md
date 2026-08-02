# API の構造

\`/kcsapi/\` のエンドポイントとレスポンスの構造を記録する。

書き方は [../README.md](../README.md) を参照。
**出典と観測日を必ず書くこと**（[C-03](../../spec/constraints.md)）。


## 一覧

| ファイル | 内容 | 最終観測 |
| --- | --- | --- |
| [overview.md](overview.md) | kcsapi 全体の概要（ホスト構成・`svdata=`・封筒構造・観測したエンドポイント一覧） | 2026-08-02 |
| [api_port_port.md](api_port_port.md) | `/kcsapi/api_port/port` — 母港情報（艦隊・遠征・入渠・資源・所属艦船） | 2026-08-02 |
| [api_start2_getdata.md](api_start2_getdata.md) | `/kcsapi/api_start2/getData` — マスタデータ一式（約 2.3 MB） | 2026-08-02 |
| [api_get_member_questlist.md](api_get_member_questlist.md) | `/kcsapi/api_get_member/questlist` — 任務一覧と進捗（**進捗は 50%/80% の段階のみ。回数は返らない**） | 2026-08-02（参照のみ・未実測） |
