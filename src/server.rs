//! A module that provides server implementation for the library.

use crate::Request;
use crate::ResponseLike;

/// The size of the buffer used to read incoming requests.
/// It's set to 8KiB by default.
pub const DEFAULT_BUFFER_SIZE: usize = 1024 * 8;

use std::io;
use std::net::ToSocketAddrs;

use smol::net::{SocketAddr, TcpListener, TcpStream};

#[cfg(feature = "tls")]
use smol::io::{AsyncRead, AsyncReadExt, AsyncWrite};

#[cfg(feature = "tls")]
use async_native_tls::{TlsAcceptor, TlsStream};

/// A TCP stream
#[cfg(not(feature = "tls"))]
pub type Stream = TcpStream;

/// A TLS stream.
#[cfg(feature = "tls")]
pub type Stream = TlsStream<TcpStream>;

#[cfg(feature = "websocket")]
use crate::ws::maybe_websocket;
#[cfg(feature = "websocket")]
use async_tungstenite::WebSocketStream;

/// A WebSocket handler type.
#[cfg(feature = "websocket")]
pub type WsHandler<S> = Option<(&'static str, WsHandlerFn<S>)>;
/// The inner function of a WebSocket handler.
#[cfg(feature = "websocket")]
type WsHandlerFn<S> = Arc<dyn Fn(WebSocketStream<S>) + Send + Sync + 'static>;

use std::future::Future;

#[cfg(feature = "websocket")]
use std::sync::Arc;

/// Single threaded listener made for simpler servers.
pub struct Server {
	/// The server's address
	addr: SocketAddr,
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
	pub fn new(
		addr: impl ToSocketAddrs,
		#[cfg(feature = "tls")] tls_acceptor: TlsAcceptor,
	) -> io::Result<Self> {
		Ok(Self {
			addr: addr.to_socket_addrs()?.next().ok_or(io::Error::new(
				io::ErrorKind::InvalidData,
				"Failed to obtain a valid address",
			))?,
			buffer_size: DEFAULT_BUFFER_SIZE,
			#[cfg(feature = "websocket")]
			ws_handler: None,
			#[cfg(feature = "tls")]
			tls_acceptor,
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
	pub fn addr(&self) -> SocketAddr {
		self.addr
	}

	/// Get the address the server is listening on as a string,
	/// formatted to be able to use it as a link.
	pub fn pretty_addr(&self) -> String {
		crate::util::format_addr(self.addr)
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
	/// See [`Server::set_buffer_size`].
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
	pub fn on_websocket<F, R>(mut self, path: &'static str, handler: F) -> Self
	where
		F: Fn(WebSocketStream<Stream>) -> R + Send + 'static + Clone + Sync,
		R: Future<Output = ()> + Send + 'static,
	{
		let real_handler: WsHandlerFn<Stream> = Arc::new(move |s: WebSocketStream<Stream>| {
			smol::spawn(handler(s)).detach();
		});

		self.ws_handler = Some((path, real_handler));
		self
	}

	/// Runs the server asynchronously.
	pub async fn run<F, T, R>(mut self, handler: F) -> io::Result<()>
	where
		F: Fn(Request) -> R + Send + 'static + Clone,
		R: Future<Output = T> + Send + 'static,
		T: ResponseLike + 'static,
	{
		let buffer_size = self.buffer_size;
		let should_insert_defaults = self.insert_default_headers;
		#[cfg(feature = "websocket")]
		let ws_handler = self.ws_handler.clone();
		let listener = TcpListener::bind(self.addr).await?;
		loop {
			let (stream, addr) = self.next_stream(&listener).await;
			smol::spawn(Self::keep_handling(
				buffer_size,
				should_insert_defaults,
				stream,
				addr,
				handler.clone(),
				#[cfg(feature = "websocket")]
				ws_handler.clone(),
			))
			.detach();
		}
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
			#[cfg_attr(not(feature = "websocket"), expect(unused_mut))]
			let mut req = match Request::read_from(&mut stream, addr, buffer_size).await {
				Ok(req) => req,
				Err(e) if e.kind() == io::ErrorKind::InvalidInput => {
					crate::response!(bad_request)
						.send_to(&mut stream)
						.await
						.ok();
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
						.await
						.ok();
					break;
				}
			};

			#[cfg(feature = "websocket")]
			match maybe_websocket(ws_handler.clone(), stream, &mut req).await {
				Err(new_stream) => {
					stream = new_stream;
				}
				_ => {
					break;
				}
			}

			let keep_alive = req.keep_alive();

			let mut response = handler(req)
				.await
				.to_response()
				.maybe_add_defaults(should_insert_defaults);

			let force_close = response
				.headers
				.get("connection")
				.map(|s| s.to_ascii_lowercase())
				== Some("close".to_string());

			if keep_alive && !force_close {
				response.set_header("connection", "keep-alive".into());
			} else {
				response.set_header("connection", "close".into());
			};

			let _ = response.send_to(&mut stream).await; // can't do much about it if it fails
		}
	}
}

// This is a workaround to avoid having to copy documentation.

impl Server {
	/// Try to accept a new incoming stream safely.
	///
	/// # Example
	/// ```rust
	/// use snowboard::Server;
	///
	/// let server = Server::new("localhost:3000").await?;
	/// match server.try_accept().await {
	///     Ok((_, ip)) => println!("new connection to {:#?}", ip),
	///     Err(_) => println!("a stream has failed to connect !"),
	/// }
	/// ```
	#[inline]
	pub async fn try_accept(&self, listener: &TcpListener) -> io::Result<(Stream, SocketAddr)> {
		let (stream, addr) = listener.accept().await?;
		self.try_accept_inner(stream, addr).await
	}

	#[cfg(not(feature = "tls"))]
	#[inline]
	/// A helper function which handles the requests done from the client.
	async fn try_accept_inner(
		&self,
		stream: TcpStream,
		addr: SocketAddr,
	) -> io::Result<(Stream, SocketAddr)> {
		stream.set_nodelay(true)?;
		Ok((stream, addr))
	}

	/// Tries to accept the request as TLS. To do so without breaking it, checks first for TLS
	/// indicators. If not found, redirects to HTTPS.
	#[cfg(feature = "tls")]
	async fn try_accept_inner(
		&self,
		mut tcp_stream: TcpStream,
		ip: SocketAddr,
	) -> io::Result<(Stream, SocketAddr)> {
		// Using `tls_acceptor` directly consumes the first 4 bytes of the stream,
		// making redirects hard (and maybe impossible) to implement. `native_tls` uses
		// different implementations (even externally) for `TlsAcceptor`, so the only
		// safe way is this.

		tcp_stream.set_nodelay(true)?;

		let mut buffer = [0; 2];
		tcp_stream.peek(&mut buffer).await?;

		if buffer == [0x16, 0x03] {
			// This looks like a TLS handshake.
			match self.tls_acceptor.accept(tcp_stream).await {
				Ok(t) => Ok((t, ip)),
				Err(_) => {
					// Continue to the next connection
					Err(io::Error::from(io::ErrorKind::ConnectionAborted))
				}
			}
		} else {
			// This doesn't look like a TLS handshake. Handle it as a non-TLS request.
			self.handle_not_tls(&mut tcp_stream).await?;
			Err(io::Error::from(io::ErrorKind::ConnectionAborted))
		}
	}

	/// Extremely simple HTTP to HTTPS redirect.
	#[cfg(feature = "tls")]
	async fn handle_not_tls<T: AsyncRead + AsyncWrite + Unpin>(
		&self,
		mut stream: T,
	) -> io::Result<()> {
		let mut buffer: Vec<u8> = vec![0; self.buffer_size];
		let length = stream.read(&mut buffer).await?;

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

		let mut res = crate::response!(
			moved_permanently,
			[],
			crate::headers! {
				"Location" => format!("https://{}{}", self.pretty_addr(), path),
				"Connection" => "keep-alive",
			}
		);

		res.send_to(&mut stream).await?;

		Ok(())
	}

	/// Waits for the next stream to be accepted.
	pub async fn next_stream(&mut self, listener: &TcpListener) -> (Stream, SocketAddr) {
		loop {
			match self.try_accept(listener).await {
				Ok(r) => return r,
				Err(e)
					if e.kind() == io::ErrorKind::ConnectionAborted
						|| e.kind() == io::ErrorKind::ConnectionReset
						|| e.kind() == io::ErrorKind::InvalidInput =>
				{
					continue;
				}
				Err(e) => {
					eprintln!("[internal server error !!] {}", e);
					eprintln!("[internal server error !!] {:#?}", e);
					continue;
				}
			}
		}
	}
}
