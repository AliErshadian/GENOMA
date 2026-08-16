import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./app/**/*.{ts,tsx}", "./components/**/*.{ts,tsx}", "./features/**/*.{ts,tsx}"],
  theme: {
    extend: {
      colors: {
        void: "#05060a",
        cyan: "#7ee0f2",
        purple: "#b48cff",
        signal: "#6b8cff",
        mutation: "#ff9a4a",
        anomaly: "#ff5d6c",
        core: "#f4f6fb",
      },
      fontFamily: {
        sans: ["var(--font-outfit)", "system-ui", "sans-serif"],
        mono: ["var(--font-ibm)", "ui-monospace", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
