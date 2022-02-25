WebGL2 port for the 7th of OpenGL Super Bible.

## Run

```bash
yarn
cargo install cargo-watch
cargo watch -i "pkg/*" -s "rm -rf ./pkg && wasm-pack build --release -d pkg"
yarn serve
```
visit [http://127.0.0.1:8080/](http://127.0.0.1:8080/) in your browser.