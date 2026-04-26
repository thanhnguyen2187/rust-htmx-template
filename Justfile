[parallel]
dev: dev-server dev-styles

dev-server:
    watchexec -r -e rs,html cargo run

dev-styles:
    tailwindcss --input ./styles/input.css --output ./static/styles.css --watch


