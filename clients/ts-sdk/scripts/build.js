import path from "node:path";
import { build as esbuild } from "esbuild";

const srcPath = path.join(process.cwd(), "src");
const buildPath = path.join(process.cwd(), "dist");

async function build() {
  return await esbuild({
    platform: "node",
    target: "node21",
    format: "esm",
    nodePaths: [srcPath],
    sourcemap: true,
    treeShaking: true,
    bundle: true,
    entryPoints: [path.join(srcPath, "index.ts")],
    outdir: buildPath,
  });
}

build();
