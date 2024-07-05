const preset = require("config/tailwind");

/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{astro,html,js,jsx,md,mdx,svelte,ts,tsx,vue}",
    "../shared/ui/**/*.{astro,html,js,jsx,md,mdx,ts,tsx}",
  ],
  presets: [preset],
};
