const { build } = require("esbuild");

build({
  entryPoints: ["src/index.tsx"],
  treeShaking: true,
  outdir: "./dist",
  sourcemap: false,
  splitting: true,
  bundle: true,
  minify: true,
  format: "esm",
  target: ["es2020"],
});
