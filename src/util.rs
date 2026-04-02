//! A module that provides code to handle the HTTP/HTTPS header method types.

use std::{fmt::Display, net::SocketAddr};

/// Any valid HTTP method.
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
	/// GET
	GET,
	/// POST
	POST,
	/// PUT
	PUT,
	/// DELETE
	DELETE,
	/// HEAD
	HEAD,
	/// OPTIONS
	OPTIONS,
	/// CONNECT
	CONNECT,
	/// PATCH
	PATCH,
	/// TRACE
	TRACE,
	/// Unknown method
	UNKNOWN,
}

impl Display for Method {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

impl From<&[u8]> for Method {
	fn from(method: &[u8]) -> Self {
		match method {
			b"GET" => Method::GET,
			b"POST" => Method::POST,
			b"PUT" => Method::PUT,
			b"DELETE" => Method::DELETE,
			b"HEAD" => Method::HEAD,
			b"OPTIONS" => Method::OPTIONS,
			b"CONNECT" => Method::CONNECT,
			b"PATCH" => Method::PATCH,
			b"TRACE" => Method::TRACE,
			_ => Method::UNKNOWN,
		}
	}
}

/// HTTP protocol version.
/// `HttpVersion::UNKNOWN` is used when the version is not specified or not valid.
/// Note that for now only HTTP/1.1 is supported.
#[cfg_attr(feature = "json", derive(serde::Serialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpVersion {
	/// HTTP/1.0
	V1_0,
	/// HTTP/1.1
	V1_1,
	/// HTTP/2.0
	V2_0,
	/// HTTP/3.0
	V3_0,
	/// Unknown version
	Other,
}

impl Display for HttpVersion {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"HTTP/{}",
			match self {
				HttpVersion::V1_0 => "1.0",
				HttpVersion::V1_1 => "1.1",
				HttpVersion::V2_0 => "2.0",
				HttpVersion::V3_0 => "3.0",
				// If the version isn't valid, and the user tries to send a response,
				// it'll just send a 1.1 response. This might cause problems, but it's
				// better than crashing.
				HttpVersion::Other => "1.1",
			}
		)
	}
}

impl From<&str> for HttpVersion {
	fn from(version: &str) -> Self {
		match version {
			"HTTP/1.0" => HttpVersion::V1_0,
			"HTTP/1.1" => HttpVersion::V1_1,
			"HTTP/2.0" => HttpVersion::V2_0,
			"HTTP/3.0" => HttpVersion::V3_0,
			_ => HttpVersion::Other,
		}
	}
}

/// Formats a socket address into something usable.
pub fn format_addr(addr: SocketAddr) -> String {
	match addr {
		SocketAddr::V4(v4) => {
			if v4.ip().is_loopback() {
				format!("localhost:{}", v4.port())
			} else {
				v4.to_string()
			}
		}
		SocketAddr::V6(v6) => {
			if v6.ip().is_loopback() {
				format!("localhost:{}", v6.port())
			} else {
				format!("{}:{}", v6.ip(), v6.port())
			}
		}
	}
}
