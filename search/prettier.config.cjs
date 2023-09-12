/* eslint-disable no-undef */
/* eslint-disable @typescript-eslint/no-unsafe-call */
module.exports = {
  plugins: [
    require("prettier-plugin-tailwindcss"),
    require("prettier-plugin-astro"),
  ],
  overrides: [
    {
      files: "*.astro",
      options: {
        parser: "astro",
      },
    },
  ],
  tabWidth: 2,
  trailingComma: "all",
};
