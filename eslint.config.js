// ESLint 設定（eslint + prettier / typescript-eslint 型情報つき）。
// リポジトリ構成の境界 3 本をここで強制する:
//   1. features/ どうしの import 禁止
//   2. shared/ は「ドメインを知らない」— bindings.ts / store/ / ipc/ / features/ / app/ を import 禁止
//   3. invoke / listen（@tauri-apps/api）は ipc/ の外で import 禁止
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import importPlugin from "eslint-plugin-import";
import prettier from "eslint-config-prettier";

const featureNames = ["game-screen", "fleet-view", "timers"];

export default tseslint.config(
  {
    ignores: [
      "dist/",
      "target/",
      "node_modules/",
      // tsc の生成物（コミットされるが手で編集しない）
      "src-tauri/injected/kcsapi-hook.js",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  // 設定ファイル自体は型情報なしで検査する
  {
    files: ["*.js", "*.ts"],
    ...tseslint.configs.disableTypeChecked,
  },
  // 注入スクリプトは専用の tsconfig で型検査する
  {
    files: ["src-tauri/injected/**/*.ts"],
    languageOptions: {
      parserOptions: {
        projectService: false,
        project: "./tsconfig.injected.json",
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },
  {
    plugins: { import: importPlugin },
    settings: {
      // TypeScript のファイルを解決できないと no-restricted-paths が発火しない
      "import/resolver": {
        node: { extensions: [".ts", ".tsx", ".js", ".jsx"] },
      },
    },
    rules: {
      "import/no-restricted-paths": [
        "error",
        {
          zones: [
            // 境界 1: features どうしは import しない（共有は shared/ へ）
            ...featureNames.map((name) => ({
              target: `./src/features/${name}`,
              from: "./src/features",
              except: [`./${name}`],
              message: `features どうしは import しない`,
            })),
            // features は app を import しない（合成は app 側の仕事）
            {
              target: "./src/features",
              from: "./src/app",
              message: "features から app を import しない",
            },
            // 境界 2: shared/ はドメインを知らない
            ...["./src/features", "./src/app", "./src/store", "./src/ipc", "./src/bindings.ts"].map(
              (from) => ({
                target: "./src/shared",
                from,
                message: "shared/ はドメインを知らないコードだけを置く",
              }),
            ),
          ],
        },
      ],
    },
  },
  // 境界 3: Rust との通信は ipc/ に集約する
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/ipc/**", "src/bindings.ts"],
    rules: {
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              message: "invoke は src/ipc/ 経由で使う",
            },
            {
              name: "@tauri-apps/api/event",
              message: "listen は src/ipc/ 経由で使う",
            },
          ],
        },
      ],
    },
  },
  prettier,
);
