//! A module that provides code to handle the websocketing funtionality of the server-client.

use std::collections::HashMap;

use crate::server::WsHandler;
use crate::{headers, Request};

use base64::engine::general_purpose::STANDARD as BASE64ENGINE;
use base64::Engine;

use async_tungstenite::tungstenite::protocol;
use async_tungstenite::WebSocketStream;
use smol::io::{AsyncRead, AsyncWrite};

use sha1::{Digest, Sha1};

/// Builds the handshake headers for a WebSocket connection.
fn build_handshake(sec_key: &String) -> HashMap<&'static str, String> {
	let mut sha1 = Sha1::new();
	sha1.update(sec_key.as_bytes());
	sha1.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
	let accept_value = BASE64ENGINE.encode(sha1.finalize());

	headers! {
		"Upgrade" => "websocket",
		"Connection" => "Upgrade",
		"Sec-WebSocket-Accept" => accept_value,
	}
}

impl Request {
	/// Checks if a request is a (usable) WebSocket handshake request.
	/// Even though the protocol requests more headers, only the
	/// `Sec-WebSocket-Key` and `Upgrade` headers are checked.
	pub fn is_websocket(&self) -> bool {
		self.headers
			.get("Upgrade")
			.map(|value| value.eq_ignore_ascii_case("websocket"))
			.unwrap_or(false)
			&& self.headers.contains_key("Sec-WebSocket-Key")
	}

	/// Upgrades a request to a WebSocket connection.
	/// Returns `None` if the request is not a WebSocket handshake request.
	pub async fn upgrade<T: AsyncWrite + AsyncRead + Unpin>(
		&mut self,
		mut stream: T,
	) -> Result<WebSocketStream<T>, T> {
		if !self.is_websocket() {
			return Err(stream);
		}

		let ws_key = match self.headers.get("Sec-WebSocket-Key") {
			Some(key) => key,
			None => return Err(stream),
		};

		let handshake = build_handshake(ws_key);

		let _ = crate::response!(switching_protocols, [], handshake)
			.send_to(&mut stream)
			.await;

		Ok(WebSocketStream::from_raw_socket(stream, protocol::Role::Server, None).await)
	}
}

/// Tries to upgrade a request to a WebSocket connection, ignoring errors.
/// If upgrading succeeds, the WebSocket is passed to `self.ws_handler`.
/// Does nothing if the request is not a WebSocket handshake request.
#[cfg(feature = "websocket")]
pub async fn maybe_websocket<S: AsyncWrite + Unpin + AsyncRead + Send + 'static>(
	handler: WsHandler<S>,
	stream: S,
	req: &mut Request,
) -> Result<(), S> {
	let handler = match handler {
		Some((path, f)) if req.url.starts_with(path) => f,
		_ => return Err(stream),
	};

	match req.upgrade(stream).await {
		Ok(s) => {
			let h = handler.clone();
			h(s);
			Ok(())
		}
		Err(s) => Err(s),
	}
}
