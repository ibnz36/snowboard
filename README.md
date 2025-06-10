<div align="center">

# This project is no longer mantained. Please open an issue if you desire to continue it, ownership will be passed.

# **Snowboard 🏂**

![License](https://img.shields.io/github/license/Brian3647/snowboard)
![Build status](https://img.shields.io/github/actions/workflow/status/Brian3647/snowboard/rust.yml)
[![DeepSource](https://app.deepsource.com/gh/Brian3647/snowboard.svg/?label=active+issues&show_trend=false)](https://app.deepsource.com/gh/Brian3647/snowboard/)
[![dependency status](https://deps.rs/repo/github/Brian3647/snowboard/status.svg)](https://deps.rs/repo/github/Brian3647/snowboard)

An extremely simple (& blazingly fast) library for HTTP & HTTPS servers in Rust

[Request a feature/Report a bug](https://github.com/Brian3647/snowboard/issues)

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
    -   [**MSRV (Minimum Supported Rust Version)**](#msrv-minimum-supported-rust-version)
    -   [**Contributing**](#contributing)
    -   [**License**](#license)

</details>

## **Quick start**

To get started with Snowboard, simply add it to your `Cargo.toml` file:

```toml
[dependencies]
snowboard = "*"
```

Then, create a new Rust file with the following code:

```rust
use snowboard::{headers, response, Method, Result, Server};

fn main() -> Result {
    let data = "Hello, world!";

    let server = Server::new("localhost:8080")?;

    println!("Listening on {}", server.pretty_addr()?);

    server.run(move |mut req| {
        if req.method == Method::DELETE {
            return response!(method_not_allowed, "Caught you trying to delete!");
        }

        req.set_header("X-Server", "Snowboard");

        println!("{req:#?}");

        response!(ok, data, headers! { "X-Hello" => "World!" })
    })
}
```

And that's it! You got yourself a working server on :8080. Examples can be found in the `examples` folder.

## **Async routes**

You can use the `async` feature and `Server::run_async` to run async routes:

```toml
# Cargo.toml

[dependencies]
snowboard = { version = "*", features = ["async"] }
```

```rust
// src/main.rs
use snowboard::{Request, ResponseLike, Server, Result};
use async_std::task;
use std::duration::Duration;

async fn index(_: Request) -> impl ResponseLike {
    task::sleep(Duration::from_secs(1)).await;

    "Async works"
}

fn main() -> Result {
    Server::new("localhost:8080")?.run_async(index)
}
```

## **TLS**

Use the `tls` feature (which will also install `native-tls`) to use TLS:

```rust
use anyhow::Result;
use snowboard::{
    Identity, TlsAcceptor,
    response, Server,
};

use std::fs;

fn main() -> Result<()> {
    let der = fs::read("identity.pfx")?;
    let password = ..;
    let tls_acceptor = TlsAcceptor::new(Identity::from_pkcs12(&der, password)?)?;

    Server::new_with_tls("localhost:3000", tls_acceptor)?
        .run(|request| format!("{request:#?}"))
}
```

You can confirm it works by running `curl -k https://localhost:3000` _(the -k is needed to allow self-signed certificates)_

More info can be found in `examples/tls`.

## **Websockets**

WebSockets are easy to implement with the `websocket` feature. Example (echo server):

```rust
use std::net::TcpStream;

use snowboard::Server;
use snowboard::WebSocket;

fn handle_ws(mut ws: WebSocket) {
    while let Ok(msg) = ws.read() {
        ws.send(msg).unwrap();
    }
}

fn main() -> snowboard::Result {
    Server::new("localhost:3000")?
        .on_websocket("/ws", handle_ws)
        .run(|_| "Try `/ws`!")
}
```

## **Routing**

Routing can be handled easily using the `Url` struct:

```rust
use snowboard::{response, Request, ResponseLike, Result, Server};

fn router(req: Request) -> impl ResponseLike {
    // /{x}
    match req.parse_url().at(0) {
        Some("ping") => response!(ok, "Pong!"),
        Some("api") => response!(not_implemented, "👀"),
        None => response!(ok, "Hello, world!"),
        _ => response!(not_found, "Route not found"),
    }
}

fn main() -> Result {
    Server::new("localhost:8080")?.run(router);
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

fn main() -> snowboard::Result {
    Server::new("localhost:8080")?.run(|req| -> Result<Value, Response> {
        let example: Example = req.force_json()?;

        Ok(serde_json::json!({
            "number_plus_one": example.number + 1
        }))
    });
}
```

```rust
use snowboard::Server;

fn main() -> snowboard::Result {
    Server::new("localhost:3000")?.run(|r| {
        serde_json::json!({
            "ip": r.ip(),
            "url": r.parse_url(),
            "method": r.method,
            "body": r.text(),
            "headers": r.headers,
		})
	})
}
```

`force_json` returns a result of either the parsed JSON or a bad request response. If you want to handle the error yourself, use `json` instead.

### **ResponseLike**

Snowboard's `ResponseLike` is designed to work with pretty much anything, but it wont by default with certain cases like `maud`'s `html!` macro. If you happen to use a lot a crate that doesn't work with Snowboard, please open an issue, pr or implement `ResponseLike` for it:

```rust
use snowboard::{Response, ResponseLike, Server};

struct Example {
    num: usize,
}

impl ResponseLike for Example {
    fn to_response(self) -> Response {
        snowboard::response!(ok, self.num.to_string())
    }
}

fn main() -> snowboard::Result {
    Server::new("localhost:8080")?
        .run(|_| Example { num: 5 });
}
```

## **MSRV (Minimum Supported Rust Version)**

The MSRV is 1.60.0, but it might change (lower or higher) depending on which features are enabled.

## **Contributing**

Check [CONTRIBUTING.md](CONTRIBUTING.md) for a simple guide on how to help the project.

## **License**

This code is under the MIT license that can be found at [LICENSE](./LICENSE)
