import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";
import node from "@astrojs/node";

import solidJs from "@astrojs/solid-js";

export default defineConfig({
  integrations: [tailwind(), solidJs()],
  output: "server",
  adapter: node({
    mode: "standalone",
  }),
});
