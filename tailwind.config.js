/** @type {import('tailwindcss').Config} */
module.exports = {
    content: [
        "./src/**/*.rs",
        "./index.html",
    ],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                // Monokai Pro (Dark) - Classic filter
                monokai: {
                    bg: "#2d2a2e",
                    bgLight: "#353236",
                    bgLighter: "#403e41",
                    bgHighlight: "#4a474b",
                    fg: "#fcfcfa",
                    fgMuted: "#939293",
                    fgDim: "#727072",
                    border: "#5b595c",
                    // Accent colors
                    red: "#ff6188",
                    orange: "#fc9867",
                    yellow: "#ffd866",
                    green: "#a9dc76",
                    blue: "#78dce8",
                    purple: "#ab9df2",
                    // UI colors
                    selection: "#49473b",
                    cursor: "#fcfcfa",
                },
                // Monokai Light
                monokaiLight: {
                    bg: "#f8f8f2",
                    bgLight: "#f1f1eb",
                    bgDark: "#e8e8e2",
                    bgHighlight: "#d8d8d2",
                    fg: "#272822",
                    fgMuted: "#595959",
                    fgDim: "#7d7d7d",
                    border: "#c0c0b8",
                    // Accent colors (slightly darker for light bg)
                    red: "#e03e5e",
                    orange: "#d97130",
                    yellow: "#c4960a",
                    green: "#6f9f3f",
                    blue: "#3ea8b5",
                    purple: "#8a72c1",
                },
            },
        },
    },
    plugins: [],
};
