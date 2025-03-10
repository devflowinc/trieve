const { context, build } = require("esbuild");

const main = async () => {
  const ctx1 = await context({
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
  const vanillaJsBuild = await context({
    entryPoints: ["src/vanilla/index.tsx"],
    treeShaking: true,
    outdir: "./dist/vanilla/",
    sourcemap: false,
    splitting: true,
    bundle: true,
    minify: true,
    format: "esm",
    target: ["es2020"],
  });

  Promise.all([ctx1.watch(), vanillaJsBuild.watch()]);
};

main();
