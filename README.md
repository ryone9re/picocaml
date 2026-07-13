# picocaml

`subsetOf (subsetOf OCaml)`

## Commands

### REPL

```sh
cargo run --bin repl
```

## Browser playground

The browser playground lets you inspect type inference, ASTs, and evaluation results. The `lab` binary builds the WebAssembly package, starts a local server, and opens the playground in your browser.

Install the wasm target and `wasm-bindgen-cli` once:

```sh
rustup target add wasm32-unknown-unknown
version=$(cargo metadata --locked --format-version 1 \
  | jq -r '.packages[] | select(.name == "wasm-bindgen") | .version')
cargo install wasm-bindgen-cli --version "$version" --locked
cargo run -p picocaml-playground --bin lab
```

The browser opens automatically. If it does not, open <http://localhost:8000> manually.

The static frontend is located in `playground/web/`. GitHub Actions builds the Rust interpreter to WebAssembly and deploys it to GitHub Pages.

In the repository settings, select `GitHub Actions` as the Pages source under Settings → Pages, then push to `main`.
