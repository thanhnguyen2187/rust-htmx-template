module.exports = {
  content: ["./src/**/*.rs", "./templates/**/*.html"],
  theme: {
    extend: {},
  },
    plugins: [
      require('@tailwindcss/typography'),
      require("daisyui"),
    ],
}
