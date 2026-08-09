// features を合成する唯一の場所。
// パネルの配置・境界のドラッグ・通知の常駐はここに実装する。
// 各 features は互いを知らず、この層だけが組み合わせを知る。

export function App() {
  // 外部仕様（docs/spec/external/）の確定後に各 features のパネルを配置する
  return <main>Harubridge</main>;
}
