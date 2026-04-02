# examples/ws

This example implements `tungstenite` to a `snowboard` server, creating a websocket echo server.

## Usage

```sh
$ cargo run --example ws --features websocket
```

## Test

```sh
$ websocat ws://localhost:3000/ws
```
