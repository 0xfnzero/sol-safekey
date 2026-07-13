import { FlatCompat } from "@eslint/eslintrc";
import { dirname } from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const compat = new FlatCompat({
  baseDirectory: __dirname,
});

const eslintConfig = [
  ...compat.extends("next/core-web-vitals", "next/typescript"),
  {
    ignores: [
      ".next/**",
      ".next-build/**",
      "out/**",
      "build/**",
      "target/**",
      "src-tauri/target/**",
      "next-env.d.ts",
      "*.config.js",
      "scripts/**/*.cjs",
    ],
  },
];

export default eslintConfig;
