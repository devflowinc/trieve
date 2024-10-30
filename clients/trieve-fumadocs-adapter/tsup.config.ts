import { defineConfig } from "tsup";

export default defineConfig({
  format: "esm",
  entry: [
    "src/app/components/dialog/search-trieve.tsx",
    "src/search/client/trieve.ts",
    "src/search/client/client.ts",
  ],
  external: [
    "fumadocs-core",
    "fumadocs-ui",
    "trieve-ts-sdk",
    "next",
    "react",
    "react-dom",
  ],
  target: "es2020",
  dts: true,
});
