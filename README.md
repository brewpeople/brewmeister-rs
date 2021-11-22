# Brewmeister

Brewmeister is the controller part of the Brewslave to execute a beer brewing
recipe. Besides setting target temperatures and holding them for a given amount
of time, it will also record temperatures and states for archival reasons.

Brewmeister is comprised of the serial communication [comm crate](./comm)
talking to the Brewslave, an Axum server [backend crate](./backend) providing a
REST interface to read current and modify target temperatures as well as a
[frontend crate](./frontend) providing a WASM module to visualize and modify the
current state in the browser.
