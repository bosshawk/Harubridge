# ADR-0025: 時刻の扱い —— 壁時計と単調増加時計の使い分け

- ステータス: **Accepted**
- 日付: 2026-08-03
- 決定者: プロジェクトオーナー
- 関連: [ADR-0016](0016-tech-stack.md)（Tauri + Rust）,
  [ADR-0018](0018-dependencies.md)（`tokio` / `notification` プラグイン）,
  [ADR-0021](0021-data-persistence.md)（C 区分は永続化しない）,
  [docs/kancolle/rules/timers.md](../kancolle/rules/timers.md),
  [docs/spec/external/timers.md](../spec/external/timers.md),
  [C-01 / C-03](../spec/constraints.md)

## 背景と課題

C 区分（時間管理・通知、FR-020〜FR-027）は Must を含む中核機能で、
そのすべてが「いま何時か」に依存する。ここを決めずに実装すると、
**ノート PC を閉じて開いたら通知が出ない**という形で壊れる。しかもその壊れ方は
macOS と Windows で異なる（後述）。実装に入る前に基準を 1 つに固定する。

### 前提として既に決まっていること（本 ADR は覆さない）

1. **ゲームは完了予定時刻を UNIX ミリ秒の絶対時刻で返す。**
   遠征は `api_mission[2]`、入渠・建造は `api_complete_time`
   （[timers.md](../kancolle/rules/timers.md)）。残り時間としては来ない。
   したがってアプリが行う演算は本質的に
   **「サーバが返した絶対時刻 − 現在時刻」**の 1 種類だけである。
2. **その絶対時刻はサーバ（JST）の時計に基づく。** 端末の時計とはずれうる。
3. **ずれは補正しない。**
   [external/timers.md](../spec/external/timers.md) が
   「端末の時計がずれている場合、表示と通知も同じだけずれる。アプリは時計のずれを補正しない」
   （E-05）と定めている。
4. 通知は OS の通知だけで、アプリが前面にあるかに依らず出す。
   スリープ等で完了時刻を過ぎてから検知した場合も
   **「アプリが起動していた限り 1 回だけ出す（出るタイミングは遅れる）」**と定められている。
5. **完了時刻を根拠にゲームへ何かを送ることはしない**（[C-01](../spec/constraints.md)）。
   時刻に反応してよいのは OS 通知と画面表示だけである。

### 決める必要があること

- 残り時間の算出と通知の発火を、**壁時計**（`std::time::SystemTime`）に載せるか、
  **単調増加時計**（`std::time::Instant`）に載せるか。
- 通知の発火方式。時刻を指定してタイマーを仕掛けるか、周期的に比較するか。
- タイムゾーン。ゲーム側の周期的な境界（任務のリセットなど）は
  端末のタイムゾーン設定と独立に JST で決まる。
- 時計がずれていることをユーザーに伝えるか。

### 調べた事実

#### `Instant` はスリープ中に進むとは限らず、しかも OS で挙動が正反対である

Rust の標準ライブラリが明記している（2026-08-03 参照）:

> As part of this non-guarantee it is also not specified whether system suspends count as
> elapsed time or not. The behavior varies across platforms and Rust versions.
> —— <https://doc.rust-lang.org/std/time/struct.Instant.html>

同ページの "Underlying System calls" によれば、`Instant::now()` が使うのは
**Darwin では `clock_gettime(CLOCK_UPTIME_RAW)`、Windows では `QueryPerformanceCounter`** である。
この 2 つはスリープの扱いが逆になる。

| | 使う API | スリープ中に進むか | 出典（2026-08-03 参照） |
| --- | --- | --- | --- |
| macOS | `clock_gettime(CLOCK_UPTIME_RAW)` | **進まない** | `clock_gettime(3)`: "clock that increments monotonically, in the same manner as CLOCK_MONOTONIC_RAW, but that **does not increment while the system is asleep**" <https://keith.github.io/xcode-man-pages/clock_gettime.3.html> |
| Windows | `QueryPerformanceCounter` | **進む** | MS Learn: "returns the total number of ticks that have occurred since the Windows operating system was started, **including the time when the machine was in a sleep state such as standby, hibernate, or connected standby**" <https://learn.microsoft.com/en-us/windows/win32/sysinfo/acquiring-high-resolution-time-stamps> |

つまり `Instant` を残り時間の基準にすると、**同じコードが macOS では
「スリープしていた分だけ残り時間が減らない」、Windows では「正しく減る」**という
プラットフォーム依存の挙動になる。
`Instant` が `CLOCK_UPTIME_RAW` を使っていることを問題視する
[rust-lang/rust#87906](https://github.com/rust-lang/rust/issues/87906) は
2026-08-03 時点で open のままであり、**近い将来に揃うという前提は置けない。**

#### `tokio` のタイマーは `Instant` に乗っている

`tokio::time::Instant` のドキュメントは "This type wraps the inner `std` variant" と明記する
（<https://docs.rs/tokio/latest/tokio/time/struct.Instant.html>、2026-08-03 参照）。
したがって `tokio::time::sleep` / `sleep_until` は上の制約をそのまま受け継ぐ。
**「完了時刻ちょうどに起きる」タイマーを tokio で仕掛けても、macOS ではスリープ分だけ遅れる。**

#### `tauri-plugin-notification` の予約通知はデスクトップで機能しない

`NotificationBuilder::schedule()` と `Schedule` / `ScheduleEvery` / `ScheduleInterval` は
API として存在する。しかしデスクトップ実装
`plugins/notification/src/desktop.rs`（`tauri-apps/plugins-workspace` の `v2` ブランチ、
2026-08-03 に取得）は `title` / `body` / `icon` / `sound` だけを読んでおり、
ファイル中に `schedule` という文字列が 1 つも現れない。
一方 `models.rs` の `NotificationData` には `schedule: Option<Schedule>` が存在する。

**すなわちデスクトップでは指定してもエラーにならず、黙って無視されて即座に発火する。**
OS 側の予約通知機構に発火を委ねる設計は取れない。

#### 表示側のタイマーも当てにできない

Chromium は非表示のページのタイマーを間引く。通常の間引きで「1 秒に 1 回」、
条件が揃った集約的な間引きでは **「1 分に 1 回」**まで落ちる
（<https://developer.chrome.com/blog/timer-throttling-in-chrome-88>、2026-08-03 参照）。
Windows の WebView2 は Chromium 系であり、この影響を受ける。
**UI 側の `setInterval` を通知の発火源にしてはならない。**

#### 既存の専ブラ 3 実装は、いずれも壁時計 + 1 秒ポーリングである

実際のソースを取得して確認した（各リポジトリの default branch の HEAD、2026-08-03）。

| 観点 | KancolleSniffer (Apache-2.0) | ElectronicObserver (MIT) | poi |
| --- | --- | --- | --- |
| 時計 | `DateTime.Now` のみ | `DateTime.Now` / `UtcNow` | `Date.now()` のみ |
| **単調時計** | **使わない** | **使わない** | **使わない** |
| 周期 | WinForms Timer 1000 ms | WinForms Timer 1000 ms | `setTimeout` 1000 ms の自己再帰 |
| 発火方式 | ポーリング + `Prev < 境界 <= Now` のエッジ検出 | ポーリング + 一度きりフラグ + 閾値比較 | ポーリング + 「同じ完了時刻では二度出さない」 |
| 時刻指定タイマー | **無し** | **無し** | **無し**（周期は 1 秒のまま） |
| 残り時間 | 毎 tick `end - Now` を計算し直す | 同左 | 同左 |
| **サーバ時刻の補正** | **していない** | **していない** | **していない** |
| 任務リセット 05:00 の基準 | **端末ローカルの 5 時** | **JST 固定**（`UtcNow + 9h`） | **UTC+4 へずらして TZ 非依存に計算** |

参照した主なファイル:

- KancolleSniffer: `KancolleSniffer/Model/AlarmTimer.cs`（`CheckAlarm` のエッジ検出）,
  `KancolleSniffer/Util/TimeStep.cs`（`Now = DateTime.Now` のみ）,
  `KancolleSniffer/Main.cs`（`new Timer {Interval = 1000}`）,
  `KancolleSniffer/Model/QuestInfo.cs`（`_now.Date.AddHours(5)` = **ローカルの 5 時**）
- ElectronicObserver: `ElectronicObserver/Utility/Mathematics/DateTimeHelper.cs`
  （`FromAPITime` は `ToLocalTime()`、一方 `GetJapanStandardTimeNow()` は `DateTime.UtcNow + 9h`）,
  `ElectronicObserver/Utility/SystemEvents.cs`（全ウィンドウ・全通知が 1 本の 1 秒 tick にぶら下がる）,
  `ElectronicObserver/Data/QuestManager.cs`（リセット判定は JST 固定側を使う）
- poi: `views/components/main/parts/countdown-timer.tsx`
  （`completeTime - Date.now()` を毎 tick 計算。`document.hidden` の間は `requestAnimationFrame` を避ける）,
  `views/services/scheduler.ts`, `views/redux/info/quests/time.ts`

poi の `views/services/scheduler.ts` は、冒頭コメントで理由を明示している:

> Scheduler: Schedule one-time or interval tasks, without worrying the problem
> that setTimeout pauses when the computer falls asleep.

**3 実装とも、サーバ時刻との同期（`Date` ヘッダの利用など）を一切していない。**
「端末の時計がずれていればそのままずれる」は、この分野で既に一般的な割り切りである。

任務のリセットについては **設計が割れている**。KancolleSniffer は端末ローカルの 5 時で
判定しており、JST 以外の環境では境界がずれる。poi はコメントで
「quests refresh at 5:00 Japan Time (UTC+9), equivalent to 0:00 UTC+4」と述べ、
UNIX ミリ秒のまま 4 時間ずらして計算することで端末のタイムゾーンから切り離している。
ElectronicObserver も JST 固定だが、`TimeZoneInfo.Local.BaseUtcOffset` を使っているため
**端末が夏時間の地域だと 1 時間ずれる**（`GetUtcOffset(time)` ではない）。
**ローカル時刻を経由して固定境界を求めると、こうした事故が起きる。**

## 決定

**1. 時刻の基準は壁時計（UNIX エポックからの絶対時刻）だけとする。**
`std::time::SystemTime` を唯一の時刻源とし、
**`Instant` を「時刻」や「残り時間」の意味を持つ用途に使わない。**
（プロファイリングやレート制限のように、区間の長さだけを見て絶対時刻と突き合わせない
用途に `Instant` を使うことは妨げない。）

**2. 残り時間は、どこでも「絶対時刻 − 現在時刻」で毎回導出する。**
カウンタを保持して 1 秒ずつ減算しない。Rust コアも UI も同じ式で計算する。
UI が持つのは完了予定時刻（UNIX ミリ秒）であって残り秒数ではない。

**3. 通知は Rust コアの周期タスクが発火する。予約タイマーを仕掛けない。**
`tokio` の周期タスクが毎秒、保持している完了予定時刻と現在の壁時計を比較し、
**「完了時刻 ≤ 現在時刻」かつ「まだ通知していない」ものを発火する。**
事前通知も同じで、比較する閾値が `完了時刻 − N 分` に変わるだけである。
この方式は状態として **「通知済みフラグ」しか要求しない**。前回 tick の時刻も、
経過時間の積算も持たない。

**4. タイムゾーンは 2 つの用途を分ける。**

| 用途 | 基準 | 理由 |
| --- | --- | --- |
| 完了時刻の表示（`HH:MM` / `M/D HH:MM`） | **端末のローカルタイムゾーン** | [external/timers.md](../spec/external/timers.md) が「タイムゾーンは OS の設定に従う」と定めている |
| ゲーム側の周期的な境界の判定（任務のリセット等） | **JST 固定（UTC+09:00）** | 境界はサーバ側で決まっており、端末の設定と無関係。端末を UTC にしても任務は日本時間で切り替わる |

現行の JST は UTC+09:00 の固定オフセットで夏時間の切り替えが無いため、
**タイムゾーンデータベース（`chrono-tz` 等）は要らない。** 固定オフセットで足りる。

**JST 固定の境界は、UNIX 時刻に +9 時間して算術で求める。
端末のローカル時刻に変換してから境界を組み立ててはならない。**
ElectronicObserver はローカル時刻を経由したために夏時間の地域で 1 時間ずれる形になっており、
KancolleSniffer は端末ローカルの 5 時で判定している（上記）。同じ轍を踏まない。

**5. 時計のずれをユーザーに提示しない。**
E-05（補正しない）を維持し、警告も出さない。
**補正しないと決めている以上、ずれを知らせてもユーザーが取れる行動が無い**からである。
OS の時計を直すよう促すことはできるが、それは本アプリが担う役割ではない。

なお、サーバ時刻を知る手段があるかどうかは**本決定の理由に含めない**。
ゲームサーバが `Date` ヘッダを返すか、それを注入スクリプトから読み取れるかは未確認であり
（案 C の補足）、仮に読めると判明しても、E-05 が生きている限り決定 5 は変わらない。

## 検討した選択肢

### 案 A: 壁時計だけを使い、通知は毎秒のポーリングで発火する（採用）

- 概要: 上記の決定のとおり。`SystemTime` を唯一の時刻源とし、
  1 秒周期のタスクが「完了時刻 ≤ 現在時刻 かつ 未通知」を評価する。
- 利点:
  - サーバが返す絶対時刻と**同じ座標系**にいるため、そのまま引き算できる。
    変換も基準点の保持も要らない。
  - **スリープの扱いに OS 差が出ない。** 復帰後の最初の 1 tick で跨ぎが検出され、
    通知が「遅れて 1 回だけ」出る。これは
    [external/timers.md](../spec/external/timers.md) が既に定めた振る舞いと一致する。
  - 時刻が未来に飛べば、跨いだものがその tick でまとめて 1 回ずつ出る。
    過去に飛んでも通知済みフラグにより再発火しない。
    **どちらの向きのジャンプでも「1 事象 1 通知」が保たれる。**
  - 状態が「通知済みフラグ」だけで済む。ADR-0021 が
    「C 区分は永続化が要らない」としているのと整合する。
  - **既存の専ブラ 3 実装がいずれもこの形である**（上記）。
    前例が薄い方式ではない（poi だけは発火場所が UI 側である点が異なる）。
- 欠点:
  - 端末の時計がずれていれば、表示も通知もそのままずれる。
    これは E-05 として**既に受け入れられている**。
  - 毎秒 1 回、保持している完了時刻を走査する。件数は最大でも
    遠征 3 + 入渠 4 + 建造 4 の 1 桁台であり、負荷は無視できる。
  - 発火の粒度がポーリング周期（最悪 1 秒の遅れ）に律速される。
    分単位で表示する本機能では問題にならない。

### 案 B: 単調時計を基準にし、起点で壁時計と対応付ける

- 概要: 起動時に `(Instant, SystemTime)` の組を取り、以後の残り時間は `Instant` で刻む。
  壁時計のジャンプの影響を受けないようにする。
- 利点: NTP 補正・手動の時刻変更・サマータイムで表示が飛ばない。
- 欠点: **macOS ではスリープ中に `Instant` が進まないため、蓋を開けた直後の残り時間が
  実際より多く残っているように見え、通知も出ない。** Windows では進むため、
  同じコードが 2 つの OS で違う意味になる。
  std 自身が「サスペンドを経過時間に数えるかは規定しない」と明記しており、
  Rust のバージョンでも変わりうる。
- 却下理由: ノート PC の開閉が日常である本アプリで、macOS の遠征通知が丸ごと落ちる。

### 案 C: サーバ時刻とのずれを推定して補正する

- 概要: HTTP レスポンスの `Date` ヘッダなどから端末時計とサーバ時計のオフセットを求め、
  完了時刻の判定に反映する。
- 却下理由: **[external/timers.md](../spec/external/timers.md) E-05 と
  [timers.md](../kancolle/rules/timers.md) が「補正しない」を既に決めており、
  本 ADR はそれを覆す提案をしない**（覆すには仕様の変更として承認が要る）。
- 補足: 参照した 3 実装のいずれもサーバ時刻との同期をしていない（上記）。
  加えて、ゲームサーバが `Date` ヘッダを返すか、それを注入スクリプトから
  読み取れるかは**未確認**である。覆すならまずそこの実測が要る。

### 案 D: 予約通知（`tauri-plugin-notification` の `Schedule`）に発火を委ねる

- 概要: 完了時刻を指定して通知を予約し、アプリ側では何もしない。
- 却下理由: **デスクトップ実装が `schedule` を一切読んでおらず、
  黙って無視して即座に発火する**（上記「調べた事実」で実際のソースを確認）。

### 案 E: `tokio::time::sleep_until` で完了時刻にタイマーを仕掛ける

- 概要: 対象ごとに 1 本タイマーを張り、完了時刻に起きて通知する。
- 却下理由: `tokio::time::Instant` は `std::time::Instant` のラッパであり、
  **macOS ではスリープ中に進まない**ため案 B と同じ欠陥を持つ。
  加えて完了時刻は観測のたびに変わりうる（高速修復・高速建造・遠征中止）ので、
  張り直しの管理が増える。

### 案 F: 通知を UI（WebView）側の `setInterval` で発火する

- 概要: 残り時間の毎秒更新をしている UI が、0 に達したときに通知する。
  poi（Electron）は実際にこの形で、`document.hidden` の間は
  `requestAnimationFrame` を避けるという回避策を入れている。
- 却下理由: **[architecture.md](../spec/architecture.md) の
  「UI は Rust コアから受け取った状態のみを描画する」に反する。**
  加えて、非表示時のタイマーが最大 1 分に 1 回まで間引かれる環境があり
  （Chromium）、回避策が要る時点で Rust 側に置くほうが単純である。

## 決め手

**スリープをまたいでも OS 差なく動くことを得るために、時刻ジャンプに対する頑健さを捨てた。**
捨てたほうは E-05 として既に受け入れ済みであり、新たに何かを失う判断ではない。

## 影響

### 実装への影響

- Rust コアが**唯一の時刻の権威**になる。時刻を扱う入口を `SystemTime::now()` に一本化する。
- 通知の発火は Rust コアの 1 秒周期タスクに置く。UI は発火に関与しない。
- UI へ渡すのは**完了予定時刻（UNIX ミリ秒）**であり、残り秒数ではない。
- ゲーム側の周期境界の判定に、固定オフセット +09:00 を使う。
  **端末のタイムゾーンを読んで境界を決める実装をしてはならない。**
- テストのために、時刻の取得を差し替え可能にしておく必要がある
  （具体的な形はコードが正 —— [ADR-0008](0008-code-as-source-of-truth.md)）。
- [C-01](../spec/constraints.md) の再確認: 周期タスクはローカルの状態を見て
  OS 通知を出すだけで、**ネットワークに一切触れない。**

### ドキュメントへの影響

- **[external/timers.md](../spec/external/timers.md) の未解決事項の 1 つが解消する。**
  「完了時刻が絶対時刻として得られるか、残り時間として得られるか」は
  [timers.md](../kancolle/rules/timers.md) により**絶対時刻**と確認済みである。
  当該行の削除は仕様の変更にあたるため承認を得たうえで、本 ADR の承認と同時に行った。
- 任務のリセット時刻（JST 固定であること、および正確な時刻）は
  **`docs/kancolle/` に記録が無い。** FR-050 の実装前に記録が要る（下記）。
- 時刻の扱いを横断規約として [docs/guidelines/](../guidelines/) に落とすかは別途判断する。
  本 ADR で足りるなら書かない（二重管理を避ける）。

### 取り消す場合のコスト

**低い。** 時刻の取得点を一本化しておけば差し替えられる。
通知の発火方式を予約型に変えることも、状態が「通知済みフラグ」だけなので影響が閉じている。
ただし **UI へ残り秒数を渡す形に変えた後**では、UI 側にも変更が波及する。

### 見直しの条件

本 ADR は実装前の判断であり、**実装中に次のいずれかが起きたら新しい ADR で見直す**。

- **スリープ・休止から復帰した最初の tick で、跨いだ完了時刻が検出されない。**
  決定 1 と決定 3 の前提そのものが崩れる。
- **時計が前後にジャンプしたときに「1 事象 1 通知」が破れる**
  （通知が二重に出る、または落ちる）。案 A の利点として挙げた性質が成り立っていない。
- **通知の発火が分単位の表示に間に合わない**遅れを実測で示す。

一方、次は本 ADR の見直しにあたらない。**該当する場所で片付ける。**

- ポーリング周期が省電力・CPU 使用率の面で重い → 周期を変えるだけで決定は変わらない
  （未解決事項のとおり実装で決める）。
- 端末の時計のずれが実用上つらい → 覆すべきは
  [external/timers.md](../spec/external/timers.md) の E-05 であり、本 ADR ではない。
- Rust 側の暦計算にライブラリが要ると判明した → 依存の追加として
  [ADR-0018](0018-dependencies.md) 側で扱う。

## 未解決事項

- `TODO(要検証)`: **任務のリセット時刻を `docs/kancolle/` に記録すること。**
  デイリー 05:00 / ウィークリー 月曜 05:00 / マンスリー 1 日 05:00 は、
  ElectronicObserver `Data/QuestManager.cs` と poi `views/redux/info/quests/time.ts`
  （"quests refresh at 5:00 Japan Time (UTC+9)"）の 2 実装で一致するところまで確認したが、
  **`docs/kancolle/` に記録が無い。** 記録は本 ADR ではなく
  [`research-kancolle`](../../.claude/skills/research-kancolle/SKILL.md) の作業である。
  **本 ADR が決めたのは「JST 固定で判定する」までで、時刻の値そのものではない。**
- `TODO(要検証)`: **クォータリーの区切り。** poi は
  「`Tanaka quarter starts from Feb`」として **2 月始まり**で扱い、
  ElectronicObserver は 1 月始まりで扱っている（`IsCrossedQuarter(..., 0, 1, 5, 0, 0)`）。
  **2 実装が食い違っている。** FR-050 の実装前に確認が要る。
- `TODO(要検証)`: **macOS（WKWebView）のタイマー間引き。**
  Chromium 側は出典を確認したが、WKWebView 側は確認できていない。
  ただし決定 3（発火は Rust 側）により、判明しても結論は変わらない。
- `TODO(未確定)`: **日付・時刻ライブラリの選定。** [ADR-0018](0018-dependencies.md) に含まれていない。
  固定オフセットの扱いは `chrono` / `time` / `jiff` のいずれでも可能であり、
  本 ADR の結論を左右しないため、依存の追加として別途決める。
  **ただし本 ADR の決定により、選定の範囲は次のとおり縮んでいる。**
  決定 2 により UI が持つのは完了予定時刻の UNIX ミリ秒であり、
  ローカルタイムゾーンでの表示整形は UI 側で完結する。
  決定 4 により Rust 側は UNIX 時刻に +9 時間した算術で境界を求めるため、
  日次の境界は除算で足りる。**Rust 側で暦の計算が要るのは週初・月初・四半期の境界だけ**であり、
  ライブラリを入れるかどうかもそこだけで判断してよい。
  ADR を新たに起こすほどの判断でなければ [ADR-0018](0018-dependencies.md) への追記で足りる。
- `TODO(未確定)`: **ポーリング周期を 1 秒でよいか。**
  NFR-006（メモリ）と省電力の観点から、ウィンドウ非表示時に周期を落とすかは別途検討する。
  落とす場合でも、事前通知の分数（既定 5 分）より十分細かい必要がある。
- `TODO(未確定)`: スリープ復帰を OS のイベントとして受け取るか。
  ポーリングで足りると判断したため現時点では不要。周期を大きく落とす場合に再検討する。
- `TODO(未確定)`: 疲労回復（FR-025）の予測は 3 分周期の**サーバ側の位相**に依存し、
  観測から推定するしかない（[fatigue.md](../kancolle/rules/fatigue.md)）。
  本 ADR は「端末の時計をどう読むか」だけを決めており、
  **位相の推定方法は別の問題**である。
