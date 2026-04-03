<div align="center">

# **Snowboard 🏂**

![License](https://img.shields.io/github/license/ibnz36/snowboard)
![Build status](https://img.shields.io/github/actions/workflow/status/ibnz36/snowboard/rust.yml)
[![DeepSource](https://app.deepsource.com/gh/ibnz36/snowboard.svg/?label=active+issues&show_trend=false)](https://app.deepsource.com/gh/ibnz36/snowboard/)
[![dependency status](https://deps.rs/repo/github/ibnz36/snowboard/status.svg)](https://deps.rs/repo/github/ibnz36/snowboard)

An extremely simple (& blazingly fast) library for HTTP & HTTPS servers in Rust

[Request a feature/Report a bug](https://github.com/ibnz36/snowboard/issues)

</div>

<details>
<summary>Table of Contents</summary>

-   [**Snowboard 🏂**](#snowboard-)
    -   [**Quick start**](#quick-start)
    -   [**Async routes**](#async-routes)
    -   [**TLS**](#tls)
    -   [**Websockets**](#websockets)
    -   [**Routing**](#routing)
    -   [**Integration**](#integration)
        -   [**JSON**](#json)
        -   [**ResponseLike**](#responselike)
    -   [**Contributing**](#contributing)
    -   [**License**](#license)

</details>

## **Quick start**

To get started with Snowboard, simply add it and `smol-potat` to your `Cargo.toml` file:

```toml
[dependencies]
snowboard = "*"
smol-potat = { version = "*", features = "auto" }
```

Then, create a new Rust file with the following code:

```rust
use snowboard::{headers, response, Server};

#[snowboard::main]
async fn main() -> snowboard::Result {
    let server = Server::new("localhost:8080")?;

    println!("Listening on {}", server.pretty_addr()?);

    server
        .run(async move |mut req| {
            println!("{req:#?}");
            response!(ok, "hello from snowboard!", headers! { "X-Hello" => "World!" })
        })
        .await
}
```

And that's it! You got yourself a working server on :8080. Examples can be found in the `examples` folder.

## **TLS**

Use the `tls` feature to create secure-connection servers:

```rust
use anyhow::Result;
use snowboard::TlsAcceptor;

use snowboard::Server;

use snowboard::smol::fs::File;

#[snowboard::main]
async fn main() -> Result<()> {
    let password = "1234";
    let idx = File::open("identity.pfx").await?;
    let tls_acceptor = TlsAcceptor::new(idx, password).await?;

    Server::new("localhost:3000", tls_acceptor)?
        .run(async move |request| format!("{request:#?}"))
        .await
}
```

More info can be found in `examples/tls`.

## **Websockets**

WebSockets are easy to implement with the `websocket` feature. Example (echo server):

```rust
use snowboard::Server;
use snowboard::{StreamExt, WebSocket};

async fn handle_ws(ws: WebSocket) {
    let (mut sender, mut reciever) = ws.split();
    while let Some(Ok(msg)) = reciever.next().await {
        let _ = sender.send(msg).await;
    }
}

#[snowboard::main]
async fn main() -> snowboard::Result {
    Server::new("localhost:8080")?
        .on_websocket("/ws", handle_ws)
        .run(async |_| "Try `/ws`!")
        .await
}
```

## **Routing**

Routing can be handled easily using the `Url` struct:

```rust
use snowboard::{response, Request, ResponseLike, Result, Server};

async fn router(req: Request) -> impl ResponseLike {
    // /{x}
    match req.parse_url().at(0) {
        Some("ping") => response!(ok, "Pong!"),
        Some("api") => response!(continue),
        None => response!(ok, "Hello, world!"),
        _ => response!(not_found, "Route not found"),
    }
}

#[snowboard::main]
async fn main() -> Result {
    Server::new("localhost:8080")?.run(router).await
}
```

## **Integration**

### **JSON**

JSON is supported with the `json` feature (serializing & deserializing):

```rust
use serde_json::Value;
use snowboard::{Response, Server};

#[derive(serde::Deserialize)]
struct Example {
    number: isize,
}

#[snowboard::main]
async fn main() -> snowboard::Result {
    Server::new("localhost:8080")?
        .run(async |req| -> Result<Value, Response> {
            let parsed: Example = req.expect_json()?;

            Ok(serde_json::json!({
                "number_plus_one": parsed.number + 1
            }))
        })
        .await
}
```

`expect_json` returns a result of either the parsed JSON or a bad request response. If you want to handle the error yourself, use the `json` function instead.

### **ResponseLike**

Snowboard's `ResponseLike` is designed to work with pretty much anything, but it wont by default with certain cases like `maud`'s `html!` macro. If you happen to use a lot a crate that doesn't work with Snowboard, you can implement `ResponseLike` for it:

```rust
use snowboard::{Response, ResponseLike, Server};

struct Example {
    num: usize,
}

impl ResponseLike for Example {
    fn to_response(self) -> Response {
        snowboard::response!(ok, format!("My favorite number is {}!", self.num))
    }
}

#[snowboard::main]
async fn main() -> snowboard::Result {
    Server::new("localhost:8080")?
        .run(async |_| Example { num: 5 })
        .await;
}
```

## **Contributing**

Check [CONTRIBUTING.md](CONTRIBUTING.md) for a simple guide on how to help the project.

## **License**

This code is under the MIT license that can be found at [LICENSE](./LICENSE)
