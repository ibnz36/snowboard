#![forbid(unsafe_code, clippy::panic)]
#![warn(clippy::cognitive_complexity, rust_2018_idioms)]
#![doc = include_str!("../README.md")]

mod macros;
mod request;
mod response;
mod server;
mod url;
mod util;

#[cfg(feature = "websocket")]
mod ws;

pub use request::Request;
pub use response::{Headers, Response, ResponseLike};
pub use server::{Server, Stream, DEFAULT_BUFFER_SIZE};
pub use url::Url;
pub use util::Method;

pub use response::DEFAULT_HTTP_VERSION as _DEFAULT_HTTP_VERSION; // we need this for macros
pub(crate) use util::HttpVersion;

#[cfg(feature = "websocket")]
/// A WebSocket connection.
pub type WebSocket = async_tungstenite::WebSocketStream<Stream>;

#[cfg(feature = "tls")]
// Re-export needed structs for `Server::new(...)` with TLS.
pub use async_native_tls::TlsAcceptor;

pub use smol;
pub use smol::stream::StreamExt;

pub use smol_potat;
pub use smol_potat::main;

/// A type alias for `std::io::Result<()>`
/// used in `Server::new()?.run(...)`.
///
/// `Server::run` returns type `!` (never) so using `Ok(())` is not needed.
pub type Result = std::io::Result<()>;
