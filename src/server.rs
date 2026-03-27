//! A module that provides server implementation for the library.

use crate::Request;
use crate::ResponseLike;

/// The size of the buffer used to read incoming requests.
/// It's set to 8KiB by default.
pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 8;

use std::{
	io,
	net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs},
};

#[cfg(feature = "tls")]
use native_tls::{TlsAcceptor, TlsStream};

/// A TCP stream
#[cfg(not(feature = "tls"))]
pub type Stream = TcpStream;

/// A TLS stream.
#[cfg(feature = "tls")]
pub type Stream = TlsStream<TcpStream>;

#[cfg(feature = "websocket")]
use crate::ws::{maybe_websocket, WebSocket};

/// A WebSocket handler function type.
#[cfg(feature = "websocket")]
pub type WsHandler<S> = Option<(&'static str, fn(WebSocket<&mut S>))>;

use std::future::Future;

/// Single threaded listener made for simpler servers.
pub struct Server {
	/// It stores the TcpListener struct.
	acceptor: TcpListener,
	/// It stores the buffer size for the Tcp requests.
	buffer_size: usize,
	/// It stores the default HTTP/HTTPS request headers.
	insert_default_headers: bool,
	/// It stores the TlsAcceptor struct when the tls feature is enabled.
	#[cfg(feature = "tls")]
	tls_acceptor: TlsAcceptor,
	#[cfg(feature = "websocket")]
	/// It stores the WebSocket configuration for the HTTP/HTTPS server.
	ws_handler: WsHandler<Stream>,
}

/// Simple rust TCP HTTP server.
impl Server {
	/// Create a new server instance.
	/// The server will listen on the given address.
	#[cfg(not(feature = "tls"))]
	pub fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
		Ok(Self {
			acceptor: TcpListener::bind(addr)?,
			buffer_size: DEFAULT_BUFFER_SIZE,
			#[cfg(feature = "websocket")]
			ws_handler: None,
			insert_default_headers: false,
		})
	}

	/// Create a new server instance with TLS.
	/// The server will listen on the given address.
	#[cfg(feature = "tls")]
	pub fn new_with_tls(addr: impl ToSocketAddrs, tls_acceptor: TlsAcceptor) -> io::Result<Self> {
		Ok(Self {
			acceptor: TcpListener::bind(addr)?,
			buffer_size: DEFAULT_BUFFER_SIZE,
			tls_acceptor,
			#[cfg(feature = "websocket")]
			ws_handler: None,
			insert_default_headers: false,
		})
	}

	/// Enables automatic insertion of default headers in responses.
	/// This includes `Server` and `Date`.
	pub fn with_default_headers(mut self) -> Self {
		self.insert_default_headers = true;
		self
	}

	/// Get the address the server is listening on.
	#[inline]
	pub fn addr(&self) -> io::Result<SocketAddr> {
		self.acceptor.local_addr()
	}

	/// Get the address the server is listening on as a string,
	/// formatted to be able to use it as a link.
	pub fn pretty_addr(&self) -> io::Result<String> {
		self.addr().map(crate::util::format_addr)
	}

	/// Set the buffer size used to read incoming requests.
	/// The default buffer size is 8KiB.
	///
	/// If you want requests to actually get parsed, the buffer size must be greater than 5,
	/// the minimum size of a "valid" HTTP request (`GET /`). The minimum size is larger on
	/// secure requests.
	///
	/// Consider using a smaller buffer size if your server
	/// doesn't require bodies in requests, and a larger one if
	/// you expect large payloads. 8KiB is a good default, tho.
	///
	/// Note that requests bigger than the buffer size will be cut off.
	pub fn set_buffer_size(&mut self, size: usize) {
		self.buffer_size = size;
	}

	/// Sets the buffer size and returns self.
	/// See [`set_buffer_size`].
	pub fn with_buffer_size(mut self, size: usize) -> Self {
		self.buffer_size = size;
		self
	}

	/// Set a handler for WebSocket connections.
	/// The handler function will be called when a WebSocket connection is received.
	///
	/// # Example
	/// ```rust
	/// use snowboard::{response, Server};
	///
	/// Server::new("localhost:8080")
	///     .expect("Failed to start server")
	///     .on_websocket("/ws", |ws| {
	///         // Handle the WebSocket connection
	///     })
	///    .run(|_| response!(ok)); // Handle HTTP requests
	///
	#[cfg(feature = "websocket")]
	pub fn on_websocket(mut self, path: &'static str, handler: fn(WebSocket<&mut Stream>)) -> Self {
		self.ws_handler = Some((path, handler));
		self
	}

	/*
	/// Runs the server asynchronously.
	pub fn run<F, T, R>(self, handler: F) -> !
	where
		F: Fn(Request) -> R + Send + 'static + Clone,
		R: Future<Output = T> + Send + 'static,
		T: ResponseLike,
	{
		#[cfg(feature = "websocket")]
		let ws_handler = self.ws_handler;

		let should_insert = self.insert_default_headers;

		// Needed for avoiding warning when compiling without the websocket feature.
		#[cfg_attr(not(feature = "websocket"), allow(unused_mut))]
		for (mut stream, mut request) in self {
			let handler = handler.clone();

			smol::spawn(async move {
				#[cfg(feature = "websocket")]
				if maybe_websocket(ws_handler, &mut stream, &mut request.clone()) {
					return;
				};

				let _ = handler(request)
					.await
					.to_response()
					.maybe_add_defaults(should_insert)
					.send_to(&mut stream);
			})
			.detach();
		}

		unreachable!("Server::run() should never return")
	}*/

	/// Runs the server asynchronously.
	pub fn run<F, T, R>(self, handler: F) -> !
	where
		F: Fn(Request) -> R + Send + 'static + Clone,
		R: Future<Output = T> + Send + 'static,
		T: ResponseLike + 'static,
	{
		let buffer_size = self.buffer_size;
		let should_insert_defaults = self.insert_default_headers;
		#[cfg(feature = "websocket")]
		let ws_handler = self.ws_handler;
		for (stream, addr) in self {
			smol::spawn(Self::keep_handling(
				buffer_size,
				should_insert_defaults,
				stream,
				addr,
				handler.clone(),
				#[cfg(feature = "websocket")]
				ws_handler,
			))
			.detach();
		}

		unreachable!("should always be listening !")
	}

	/// Maintains a stream open for requests.
	async fn keep_handling<F, T, R>(
		buffer_size: usize,
		should_insert_defaults: bool,
		mut stream: Stream,
		addr: SocketAddr,
		handler: F,
		#[cfg(feature = "websocket")] ws_handler: WsHandler<Stream>,
	) where
		F: Fn(Request) -> R + Send + 'static,
		R: Future<Output = T> + Send + 'static,
		T: ResponseLike,
	{
		loop {
			#[cfg_attr(not(feature = "websocket"), allow(unused_mut))]
			let mut req = match Request::read_from(&mut stream, addr, buffer_size) {
				Ok(req) => req,
				Err(e) if e.kind() == io::ErrorKind::InvalidInput => {
					crate::response!(bad_request).send_to(&mut stream).ok();
					continue;
				}
				Err(e)
					if e.kind() == io::ErrorKind::BrokenPipe
						|| e.kind() == io::ErrorKind::ConnectionReset
						|| e.kind() == io::ErrorKind::UnexpectedEof =>
				{
					break;
				}
				Err(e) => {
					eprintln!("[INTERNAL ERROR] {}", e);
					crate::response!(internal_server_error)
						.send_to(&mut stream)
						.ok();
					break;
				}
			};

			#[cfg(feature = "websocket")]
			if maybe_websocket(ws_handler, &mut stream, &mut req) {
				return;
			}

			let keep_alive = req.keep_alive();

			let mut response = handler(req)
				.await
				.to_response()
				.maybe_add_defaults(should_insert_defaults);

			let force_close = match &response.headers {
				Some(h) => {
					h.get("connection").map(|s| s.to_ascii_lowercase())
						!= Some("keep-alive".to_string())
				}
				None => false,
			};

			let body_len = response.bytes.len();
			response.set_header("content-length", body_len.to_string());

			if keep_alive && !force_close {
				response.set_header("connection", "keep-alive".into());
			} else {
				response.set_header("connection", "close".into());
			};

			let _ = response.send_to(&mut stream); // can't do much about it if it fails
		}
	}
}

// This is a workaround to avoid having to copy documentation.

impl Server {
	/// Try to accept a new incoming stream safely.
	///
	/// # Example
	/// ```rust
	/// TODO
	/// ```
	#[inline]
	pub fn try_accept(&self) -> io::Result<(Stream, SocketAddr)> {
		self.try_accept_inner()
	}

	#[cfg(not(feature = "tls"))]
	#[inline]
	/// A helper function which handles the requests done from the client.
	fn try_accept_inner(&self) -> io::Result<(Stream, SocketAddr)> {
		self.acceptor.accept()
	}

	/// Tries to accept the request as TLS. To do so without breaking it, checks first for TLS
	/// indicators. If not found, redirects to HTTPS.
	#[cfg(feature = "tls")]
	fn try_accept_inner(&self) -> io::Result<(Stream, SocketAddr)> {
		// Using `tls_acceptor` directly consumes the first 4 bytes of the stream,
		// making redirects hard (and maybe impossible) to implement. `native_tls` uses
		// different implementations (even externally) for `TlsAcceptor`, so the only
		// safe way is this.

		let (mut tcp_stream, ip) = self.acceptor.accept()?;
		let mut buffer = [0; 2];
		tcp_stream.peek(&mut buffer)?;

		if buffer == [0x16, 0x03] {
			// This looks like a TLS handshake.
			match self.tls_acceptor.accept(tcp_stream) {
				Ok(t) => Ok((t, ip)),
				Err(_) => {
					// Continue to the next connection
					Err(io::Error::from(io::ErrorKind::ConnectionAborted))
				}
			}
		} else {
			// This doesn't look like a TLS handshake. Handle it as a non-TLS request.
			self.handle_not_tls(&mut tcp_stream)?;
			Err(io::Error::from(io::ErrorKind::ConnectionAborted))
		}
	}

	/// Extremely simple HTTP to HTTPS redirect.
	#[cfg(feature = "tls")]
	fn handle_not_tls<T: io::Read + io::Write>(&self, mut stream: T) -> io::Result<()> {
		let mut buffer: Vec<u8> = vec![0; self.buffer_size];
		let length = stream.read(&mut buffer)?;

		let mut path = vec![];
		let mut in_path = false;

		for byte in buffer.iter().take(length) {
			if *byte == b' ' {
				if in_path {
					break;
				} else {
					in_path = true;
					continue;
				}
			}

			if in_path {
				path.push(*byte);
			}
		}

		let path = String::from_utf8_lossy(&path).to_string();

		crate::response!(
			moved_permanently,
			[],
			crate::headers! {
				"Location" => format!("https://{}{}", self.pretty_addr().unwrap_or_default(), path),
				"Connection" => "keep-alive",
				"Content-Length" => 0
			}
		)
		.send_to(&mut stream)?;

		Ok(())
	}
}

impl Iterator for Server {
	type Item = (Stream, SocketAddr);

	fn next(&mut self) -> Option<Self::Item> {
		match self.try_accept() {
			Ok(r) => Some(r),
			// TLS errors, parse requests and cancelled connections are ignored.
			Err(e)
				if e.kind() == io::ErrorKind::ConnectionAborted
					|| e.kind() == io::ErrorKind::ConnectionReset
					|| e.kind() == io::ErrorKind::InvalidInput =>
			{
				self.next()
			}
			Err(e) => {
				// Probably an important error.
				eprintln!("Server generated error: {:#?}", e);
				self.next() // Continue anyways. We don't want to stop the server at production.
			}
		}
	}
}
