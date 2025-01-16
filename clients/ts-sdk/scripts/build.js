import path from "node:path";
import { build as esbuild } from "esbuild";

const srcPath = path.join(process.cwd(), "src");
const buildPath = path.join(process.cwd(), "dist");

const commonConfig = {
  platform: "node",
  target: "node21",
  nodePaths: [srcPath],
  sourcemap: true,
  treeShaking: true,
  bundle: true,
  entryPoints: [path.join(srcPath, "index.ts")],
};

async function build() {
  // Build ESM
  await esbuild({
    ...commonConfig,
    format: "esm",
    outdir: path.join(buildPath, "esm"),
    outExtension: { ".js": ".mjs" },
  });

  // Build CJS
  await esbuild({
    ...commonConfig,
    format: "cjs",
    outdir: path.join(buildPath, "cjs"),
    outExtension: { ".js": ".cjs" },
  });
}

build().catch((err) => {
  console.error(err);
  process.exit(1);
});
