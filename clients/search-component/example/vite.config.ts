import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [TanStackRouterVite(), react()],
  resolve: {
    alias: {
      "react-pdf-spotlight":
        "/Users/drew/programs/react-pdf-spotlight/component/src/index.tsx",
    },
  },
});
