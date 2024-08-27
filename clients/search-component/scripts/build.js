const { nodeExternalsPlugin } = require("esbuild-node-externals");
const { build } = require("esbuild");
const { dependencies, peerDependencies } = require("../package.json");

const options = {
  entryPoints: [`src/index.ts`],
  bundle: true,
  minify: true,
  sourcemap: true,
  plugins: [nodeExternalsPlugin()],
  external: [].concat.apply(
    [],
    [Object.keys(dependencies), Object.keys(peerDependencies)]
  ),
};

build({
  ...options,
  outfile: "./dist/index.esm.js",
  format: "esm",
  target: ["esnext", "node12.22.0"],
});

build({
  ...options,
  format: "cjs",
  outfile: "./dist/index.cjs.js",
  target: ["esnext", "node12.22.0"],
});

module.exports = {
  options,
};
