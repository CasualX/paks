PAKS webui viewer
=================

A vibe coded web-based viewer for PAKS files.

Developer UX
------------

Build the binary `cargo build --release --target=wasm32-unknown-unknown`.

Copy the compiled `target/wasm32-unknown-unknown/release/pakslib.wasm` to `webui/viewer/paks.wasm`.

Right click `webui/viewer/index.html` and _'Open with Live Server'_ (or similar) to run a local web server.
