const { build } = require("esbuild");
const path = require("path");

const doBuild = async () => {
  const reactBuild = await build({
    entryPoints: ["src/index.tsx"],
    treeShaking: true,
    outdir: "./dist",
    sourcemap: false,
    splitting: false,
    bundle: true,
    minify: true,
    format: "esm",
    target: ["es2020"],
    external: ["react", "react-dom"],
    define: {
      __dirname: JSON.stringify(path.resolve()),
    },
  });

  const vanillaJsBuild = await build({
    entryPoints: ["src/vanilla/index.tsx"],
    treeShaking: true,
    outdir: "./dist/vanilla/",
    sourcemap: false,
    splitting: false,
    bundle: true,
    minify: true,
    format: "esm",
    target: ["es2020"],
    define: {
      __dirname: JSON.stringify(path.resolve()),
    },
  });
};

doBuild();
