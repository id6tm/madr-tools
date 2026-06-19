# Contributing

This project uses [`proto`](https://moonrepo.dev/proto) from moonrepo to manage the local Node.js and Rust toolchains.

Install the toolchains declared in `.prototools`:

```sh
proto install
```

Run the CLI locally:

```sh
cargo run -- <command>
```

Before opening a change, run:

```sh
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```
