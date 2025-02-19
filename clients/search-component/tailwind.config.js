import defaultTheme from "tailwindcss/defaultTheme";
import plugin from "tailwindcss/plugin";
const {
  scopedPreflightStyles,
  isolateInsideOfContainer,
} = require("tailwindcss-scoped-preflight");

/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["selector", ".dark &"],
  content: ["src/**/*.{tsx,ts}"],
  plugins: [
    plugin(function ({ addVariant }) {
      addVariant("inline", () => {
        // Actually selects any child of .tv-trieve-inline-model
        return `.trieve-inline-modal &`;
      });
      addVariant("mobile-only", "@media screen and (max-width: 768px)");
    }),
    require("@tailwindcss/forms")({
      strategy: "class",
    }),
    require("tailwind-scrollbar"),
    scopedPreflightStyles({
      isolationStrategy: isolateInsideOfContainer(
        [
          "#trieve-search-component",
          "#trieve-search-modal",
          "#trieve-search-modal-overlay",
          "#open-trieve-modal",
        ],
        {
          rootStyles: true,
        },
      ),
    }),
  ],
  prefix: "tv-",
  theme: {
    fontFamily: {
      sans: ["Maven Pro", ...defaultTheme.fontFamily.sans],
      mono: defaultTheme.fontFamily.mono,
    },
    extend: {
      colors: {
        shark: {
          DEFAULT: "#202124",
          50: "#767A85",
          100: "#6D707A",
          200: "#5A5C65",
          300: "#46494F",
          400: "#33353A",
          500: "#202124",
          600: "#1B1C1F",
          700: "#161719",
          800: "#121214",
          900: "#0D0D0E",
          950: "#0A0B0C",
        },
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
        acid: {
          DEFAULT: "#D3FF19",
          50: "#F6FFD1",
          100: "#F2FFBC",
          200: "#EAFF93",
          300: "#E3FF6B",
          400: "#DBFF42",
          500: "#D3FF19",
          600: "#B5E000",
          700: "#88A800",
          800: "#5A7000",
          900: "#2D3800",
          950: "#161C00",
        },
        turquoise: {
          DEFAULT: "#00DDE7",
          50: "#A0FBFF",
          100: "#8BFAFF",
          200: "#62F8FF",
          300: "#3AF6FF",
          400: "#11F5FF",
          500: "#00DDE7",
          600: "#00A7AF",
          700: "#007277",
          800: "#003C3F",
          900: "#000607",
          950: "#000000",
        },
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
      keyframes: {
        overlayShow: {
          from: { opacity: "0" },
          to: { opacity: "1" },
        },
        contentShow: {
          from: {
            opacity: "0",
            transform: "translate(-50%, 0) scale(0.96)",
          },
          to: { opacity: "1", transform: "translate(-50%, 0) scale(1)" },
        },
      },
      animation: {
        overlayShow: "overlayShow 50ms cubic-bezier(0.16, 1, 0.3, 1)",
        contentShow: "contentShow 50ms cubic-bezier(0.17, 0.89, 0.32, 1.25)",
        "spin-border": "rotate 5s infinite linear",
      },
    },
  },
};
