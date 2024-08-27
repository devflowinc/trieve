const preset = require("config/tailwind");

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: "class",
  content: ["src/**/*.tsx"],
  presets: [preset],
  plugins: [require("@tailwindcss/forms")],
};
