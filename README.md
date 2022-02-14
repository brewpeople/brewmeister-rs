# Brewmeister

[![Rust](https://github.com/brewpeople/brewmeister-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/brewpeople/brewmeister-rs/actions/workflows/rust.yml)

Brewmeister is the controller part of the Brewslave to execute a beer brewing
recipe. Besides setting target temperatures and holding them for a given amount
of time, it will also record temperatures and states for archival reasons.

Brewmeister is comprised of

* A serial communication [comm crate](./comm) talking to the Brewslave.
* An Axum server [backend crate](./api) using the comm crate to provide a REST interface for the
  brew program management, execution and monitoring.
* A [frontend crate](./app) providing a WASM module to visualize and modify the current backend
  state in the browser.


## Running the development version

Use [rustup](https://rustup.rs/) to install a recent Rust toolchain. Then, open two shells and
start the backend server in one shell using

    $ cargo run --bin api -- [--use-mock]

Use the `--use-mock` flag if you have not connected a brewslave via the serial interface, otherwise
the server will try to open `/dev/tty/ACM0`. In another shell start the hot-reload application
using

    $ cd app
    $ trunk serve --proxy-backend=http://0.0.0.0:3000/api

By default, the server will use an in-memory SQLite database. To persist the state and override the
serial device path, create a new
`brewmeister.toml` file with

```toml
database = "/path/to/the/data.base"
device = "/dev/ttyUSB1"
```
