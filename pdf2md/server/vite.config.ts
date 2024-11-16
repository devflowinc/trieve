import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  plugins: [solidPlugin({})],
  build: {
    manifest: true,
    rollupOptions: {
      input: "client-src/index.tsx",
      output: {
        dir: "static/client",
        entryFileNames: "[name].js",
      },
    },
  },
});
