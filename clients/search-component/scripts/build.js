const { build } = require("esbuild");

const doBuild = async () => {
  const reactBuild = await build({
    entryPoints: ["src/index.tsx"],
    treeShaking: true,
    outdir: "./dist",
    sourcemap: false,
    splitting: true,
    bundle: true,
    minify: true,
    format: "esm",
    target: ["es2020"],
    external: ["react", "react-dom"],
  });

  const vanillaJsBuild = await build({
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
};

doBuild();
