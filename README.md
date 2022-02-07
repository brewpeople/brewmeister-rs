# Brewmeister

[![Rust](https://github.com/brewpeople/brewmeister-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/brewpeople/brewmeister-rs/actions/workflows/rust.yml)

Brewmeister is the controller part of the Brewslave to execute a beer brewing
recipe. Besides setting target temperatures and holding them for a given amount
of time, it will also record temperatures and states for archival reasons.

Brewmeister is comprised of the serial communication [comm crate](./comm)
talking to the Brewslave, an Axum server [backend crate](./api) providing a REST
interface to read current and modify target temperatures as well as a [frontend
crate](./app) providing a WASM module to visualize and modify the current state
in the browser.


## Running development version

Open two shells and start the backend server in one shell using

    $ cargo run --bin api -- [--use-mock]

Use the `--use-mock` if you have not connected a brewslave via serial, otherwise
the server will try to open `/dev/tty/ACM0`. In another shell start the
hot-reload application using

    $ cd app
    $ trunk serve --proxy-backend=http://0.0.0.0:3000/api
