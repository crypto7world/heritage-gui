/** @type {import('tailwindcss').Config} */
module.exports = {
  mode: "all",
  content: ["./src/**/*.{rs,html,css}", "./dist/**/*.html"],
  theme: {
    extend: {
      // colors: {
      //   primary: "rgb(var(--color-primary))",
      //   secondary: "rgb(var(--color-secondary))",
      //   front: "rgb(var(--color-text))",
      //   back: "rgb(var(--color-background))",
      // },
      animation: {
        scalebump: "scalebump 0.1s",
      },
      keyframes: {
        scalebump: {
          "0%": { transform: "scale(1);" },
          "50%": { transform: "scale(1.1);" },
          "100%": { transform: "scale(1);" },
        },
      },
    },
  },
  // darkMode: "selector",
  plugins: [require("daisyui")],
  daisyui: {
    // themes: false, // false: only light + dark | true: all themes | array: specific themes like this ["light", "dark", "cupcake"]
    themes: [
      {
        light: {
          ...require("daisyui/src/theming/themes")["light"],
          primary: "#a61d1d",
          secondary: "#09b4ba",
        },
        dark: {
          ...require("daisyui/src/theming/themes")["dark"],
          primary: "#e33f35",
          secondary: "#4ab6b7",
        },
      },
    ],
    darkTheme: "dark", // name of one of the included themes for dark mode
    base: true, // applies background color and foreground color for root element by default
    styled: true, // include daisyUI colors and design decisions for all components
    utils: true, // adds responsive and modifier utility classes
    prefix: "", // prefix for daisyUI classnames (components, modifiers and responsive class names. Not colors)
    logs: true, // Shows info about daisyUI version and used config in the console when building your CSS
    themeRoot: ":root", // The element that receives theme color CSS variables
  },
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
