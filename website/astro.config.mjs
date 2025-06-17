// @ts-check
import { defineConfig } from "astro/config";
import icon from "astro-icon";
import react from "@astrojs/react";
import keystatic from "@keystatic/astro";
import tailwindcss from "@tailwindcss/vite";

import sitemap from "@astrojs/sitemap";

import mdx from "@astrojs/mdx";

// https://astro.build/config
export default defineConfig({
  site: "https://trieve.ai",
  devToolbar: {
    enabled: false,
  },
  image: {
    domains: ["127.0.0.1"],
  },
  integrations: [
    react(),
    ...(process.env.SKIP_KEYSTATIC ? [] : [keystatic()]),
    sitemap(),
    icon(),
    mdx({
      shikiConfig: {
        theme: "nord",
      },
    }),
  ],
  vite: {
    resolve: {
      alias: {
        "@": "/src",
      },
    },
    plugins: [tailwindcss()],
    optimizeDeps: {
      exclude: ["chunk-M7RVBV2D"],
    },
  },
  redirects: {
    "/accurate-hallucination-detection-with-ner":
      "/blog/accurate-hallucination-detection-with-ner",
    "/build-hotel-voice-assistant-with-trieve-and-vapi":
      "/blog/build-hotel-voice-assistant-with-trieve-and-vapi",
    "/building-blazingly-fast-typo-correction-in-rust":
      "/blog/building-blazingly-fast-typo-correction-in-rust",
    "/building-search-for-yc-company-directory":
      "/blog/building-search-for-yc-company-directory",
    "/firecrawl-and-trieve": "/blog/firecrawl-and-trieve",
    "/history-of-hnsearch": "/blog/history-of-hnsearch",
    "/open_ai_streaming": "/blog/open_ai_streaming",
    "/pgvector-missing-features": "/blog/pgvector-missing-features",
    "/success-story-billtrack50": "/blog/success-story-billtrack50",
    "/success-story-mintlify": "/blog/success-story-mintlify",
    "/trieve-fundraise-announcement": "/blog/trieve-fundraise-announcement",
    "/trieve-self-hosting-on-vps": "/blog/trieve-self-hosting-on-vps",
    "/trieve-sitesearch-launch": "/blog/trieve-sitesearch-launch",
    "/tvi-blog": "/blog/tvi-blog",
    "/usage-based-pricing": "/blog/usage-based-pricing",
    "/we-have-a-new-js-sdk": "/blog/we-have-a-new-js-sdk",
    "/sitesearch": "/products/sitesearch",
    "/ecommerce": "/products/ecommerce",
  },
});
