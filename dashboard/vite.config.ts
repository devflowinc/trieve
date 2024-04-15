import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import md from "vite-plugin-solid-markdown";
import remarkGfm from "remark-gfm";
import rehypePrettyCode from "rehype-pretty-code";
import runtimeEnv from "vite-plugin-runtime-env";

export default defineConfig({
  plugins: [
    md({
      wrapperClasses: "prose prose-sm m-auto text-left",
      rehypePlugins: [
        [
          rehypePrettyCode,
          {
            theme: {
              dark: "one-dark-pro",
              light: "light-plus",
            },
          },
        ],
      ],
      remarkPlugins: [remarkGfm],
    }),
    solid({
      extensions: [".mdx", ".md"],
    }),
    runtimeEnv(),
  ],
});
