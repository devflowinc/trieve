/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ["./src/public/**/*.html"],
  darkMode: "class",
  theme: {
    fontFamily: {
      sans: ["Quicksand", "system-ui", "sans-serif"],
      verdana: ["Verdana", "Geneva", "sans-serif"],
    },
    extend: {
      colors: {
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
  plugins: [],
};
