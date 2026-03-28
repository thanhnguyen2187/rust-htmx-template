# HARM (HTMX, Axum, Rust, Maud) Stack Template

Full list of technologies used:

- Rust
- Axum: HTTP framework
- rusqlite + SeaQuery + serde_rusqlite: database accessing
- refinery: database migration
- Askama: HTML templating
- dotenvy: loading environment variables
- Snafu: error handling
- Tailwind CSS (with DaisyUI): styling
- (optional) Bacon + tower-livereload: live reloading

## Screenshot

![Demo](demo.png)

## Development

### Prerequisites

- Make sure that you have `rustc` and `cargo` installed (ideally by
  using [rustup](https://rustup.rs/)).
- For database migration, you should have `refinery` available (`cargo 
  install refinery_cli`).
- For Tailwind CSS and DaisyUI, you should have `tailwindcss-extra` available
  (the binary can be downloaded
  [here](https://github.com/dobicinaitis/tailwind-cli-extra/releases/)).
- For the best development experience, you should have `bacon` installed (the 
  tool can be downloaded by `cargo install bacon`).

### Starting the server

- Run without `bacon`:

```shell
cargo run
```

- Run using `bacon`:

```shell
bacon
```

- Style generation:

```shell
tailwindcss-extra --watch --output ./static/styles.css
```

The server will be available at `http://localhost:3000`.

### Migrations

TBA

### Favicon

- Generate favicons from https://favicon.io/favicon-converter/, then place them
  in `static` folder.

## Project Structure

- `src/`: contains the source code of the project
- `static/`: contains static files which are served from the root path (for
  example, `static/styles.css` will be accessed at
  `http://localhost:3000/styles.css`)
- `migrations/`: contains database migration files