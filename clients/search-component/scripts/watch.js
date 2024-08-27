const { context } = require("esbuild");
const { options } = require("./build");

const main = async () => {
  let ctx1 = await context({
    ...options,
    outfile: "./dist/index.esm.js",
    format: "esm",
    target: ["esnext", "node12.22.0"],
  });

  let ctx2 = await context({
    ...options,
    format: "cjs",
    outfile: "./dist/index.cjs.js",
    target: ["esnext", "node12.22.0"],
  });

  await ctx2.watch();
};

main();
