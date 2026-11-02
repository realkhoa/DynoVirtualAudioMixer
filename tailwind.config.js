/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        dyno: {
          dark: "#0f172a",
          darker: "#0a0f1d",
          card: "#1e293b",
          border: "#334155",
          accent: "#38bdf8",
          vuGreen: "#22c55e",
          vuYellow: "#eab308",
          vuRed: "#ef4444",
        },
      },
    },
  },
  plugins: [],
};
