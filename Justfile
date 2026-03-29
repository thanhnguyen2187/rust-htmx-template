[parallel]
dev: dev-server dev-styles

dev-server:
    watchexec -r -e rs,html cargo run

dev-styles:
    tailwindcss-extra --watch --output ./static/styles.css

