/** @type {import('tailwindcss').Config} */
module.exports = {
    content: {
        relative: true,
        files: ["*.html", "./src/**/*.rs"],
    },
    theme: {
        extend: {
            // Brand tokens — pulled verbatim from sentrix-labs/brand-kit
            // BRAND_GUIDE.md. Available as `bg-sentrix-bronze`,
            // `text-sentrix-gold`, etc. Existing zinc/amber utilities
            // still work; new components should reach for these first
            // so the visual identity tightens over time.
            colors: {
                sentrix: {
                    black: "#000000",
                    bronze: "#8A5A11",
                    gold: "#DBC17F",
                    canvas: "#0A0A0C",
                },
            },
            fontFamily: {
                mono: [
                    "ui-monospace",
                    "JetBrains Mono",
                    "Menlo",
                    "Monaco",
                    "monospace",
                ],
            },
        },
    },
    plugins: [],
};
