/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{rs,html,css}",
    "./interface/**/*.{rs,html,css}",
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'sunfire': {
          DEFAULT: '#FF8C00',
          hover: '#FFA500',
          dark: '#E67300',
        },
      },
    },
  },
  plugins: [],
}
