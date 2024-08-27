import path from "node:path";
import { build as esbuild } from "esbuild";
import { nodeExternalsPlugin } from "esbuild-node-externals";
import { readFileSync } from "node:fs";

const pkgJSON = JSON.parse(
  readFileSync(path.join(process.cwd(), "package.json"), "utf-8")
);
const buildPath = path.join(process.cwd(), "dist");

export const options = {
  entryPoints: [`src/**/index.ts`],
  outdir: buildPath,
  bundle: true,
  sourcemap: true,
  format: "esm",
  target: ["es6"],
  loader: { ".js": "jsx" },
  plugins: [nodeExternalsPlugin()],
  external: [].concat.apply(
    [],
    [Object.keys(pkgJSON.dependencies), Object.keys(pkgJSON.peerDependencies)]
  ),
};
await esbuild(options);
