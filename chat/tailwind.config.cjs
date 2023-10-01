/* eslint-disable @typescript-eslint/no-var-requires */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable no-undef */
/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: "class",
  content: ["./src/**/*.{html,js,jsx,ts,tsx}"],
  theme: {
    fontFamily: {
      sans: ["Quicksand", "sans-serif"],
    },
    extend: {
      backgroundImage: {
        "gradient-radial": "radial-gradient(var(--tw-gradient-stops))",
        "gradient-radial-t":
          "radial-gradient(farthest-side at 50% -50%, var(--tw-gradient-stops))",
        "gradient-radial-b":
          "radial-gradient(farthest-side at 50% 150%, var(--tw-gradient-stops))",
      },
      colors: {
        "cod-gray": {
          DEFAULT: "#0C0C0C",
          50: "#686868",
          100: "#5E5E5E",
          200: "#494949",
          300: "#353535",
          400: "#202020",
          500: "#0C0C0C",
          600: "#0B0B0B",
          700: "#070707",
          800: "#050505",
          900: "#040404",
        },
        mercury: {
          DEFAULT: "#E2E2E2",
          50: "#FFFFFF",
          100: "#FFFFFF",
          200: "#FFFFFF",
          300: "#FFFFFF",
          400: "#F6F6F6",
          500: "#E2E2E2",
          600: "#C6C6C6",
          700: "#AAAAAA",
          800: "#8E8E8E",
          900: "#727272",
          950: "#646464",
        },
        "mine-shaft": {
          DEFAULT: "#3C3C3C",
          50: "#989898",
          100: "#8E8E8E",
          200: "#797979",
          300: "#656565",
          400: "#505050",
          500: "#3C3C3C",
          600: "#202020",
          700: "#040404",
          800: "#000000",
          900: "#000000",
          950: "#000000",
        },
        alabaster: {
          DEFAULT: "#F8F8F8",
          50: "#FFFFFF",
          100: "#FFFFFF",
          200: "#FFFFFF",
          300: "#FFFFFF",
          400: "#FFFFFF",
          500: "#F8F8F8",
          600: "#DCDCDC",
          700: "#C0C0C0",
          800: "#A4A4A4",
          900: "#888888",
          950: "#7A7A7A",
        },
        acid: "#D3FF19",
        turquoise: "#00DDE7",
        magenta: {
          DEFAULT: "#A33EB5",
          50: "#E4C1EA",
          100: "#DDB2E5",
          200: "#CF93DA",
          300: "#C275D0",
          400: "#B557C5",
          500: "#A33EB5",
          600: "#7D308B",
          700: "#582161",
          800: "#321338",
          900: "#0C050E",
          950: "#000000",
        },
      },
    },
  },
  plugins: [
    require("tailwind-scrollbar")({ nocompatible: true }),
    require("tailwind-gradient-mask-image"),
  ],
};
