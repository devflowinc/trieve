const { context } = require("esbuild");
const { options } = require("./build");

const main = async () => {
  let ctx1 = await context({
    entryPoints: ["src/index.tsx"],
    treeShaking: true,
    outdir: "./dist",
    sourcemap: true,
    splitting: true,
    bundle: true,
    minify: false,
    format: "esm",
    target: ["es2020"],
    external: ["react", "react-dom"],
  });
};

main();
