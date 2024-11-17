/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      colors: {
        primary: "#a61d1d",
        secondary: "#09b4ba",
      },
    },
  },
  plugins: [],
  safelist: [
    {
      pattern: /((upper|lower|normal-)case|capitalize)/,
    },
    {
      pattern: /.*italic/,
    },
    {
      pattern: /.*line.*/,
    },
    {
      pattern: /text-.*/,
    },
    {
      pattern: /font-.*/,
    },
    {
      pattern: /[pm].?-.*/,
    },
    {
      pattern: /((min|max)-)?[wh]-.*/,
    },
    {
      pattern: /size-.*/,
    },
    {
      pattern: /flex.*/,
    },
    {
      pattern: /basis.*/,
    },
    {
      pattern: /grid.*/,
    },
    {
      pattern: /col.*/,
    },
    {
      pattern: /row.*/,
    },
    {
      pattern: /auto.*/,
    },
    {
      pattern: /gap.*/,
    },
    {
      pattern: /justify.*/,
    },
    {
      pattern: /content.*/,
    },
    {
      pattern: /items.*/,
    },
    {
      pattern: /self.*/,
    },
    {
      pattern: /place.*/,
    },
  ],
};
