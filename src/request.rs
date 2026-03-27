//! A module that provides code to handle https/http requests.

use std::{borrow::Cow, collections::HashMap, io, net::SocketAddr};

use crate::{Method, Url};

#[cfg(feature = "json")]
use crate::ResponseLike;

/// A server request.
/// Parses the raw request string into a more usable format.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json", derive(serde::Serialize))]
pub struct Request {
	/// The ip from the socket connection.
	pub ip: SocketAddr,
	/// Raw URL string.
	/// Use `Request::parse_url()` to get a parsed version of the URL
	pub url: String,
	/// Method used in the request. Might be Method::Unknown if parsing fails.
	pub method: Method,
	/// Body of the request, in bytes.
	/// Use [`Request::text`], [`Request::json`], or [`Request::force_json`]
	/// to get a parsed version of the body.
	pub body: Vec<u8>,
	/// Parsed headers.
	pub headers: HashMap<String, String>,
}

impl Request {
	/// Parses and creates a requeset from raw text and an ip address.
	/// Note that this does not parse the url (See [Request::url]).
	pub fn new(bytes: &[u8], ip: SocketAddr) -> Option<Self> {
		let mut words = bytes.split(|b| *b == b' ');

		let method = Method::from(words.next()?);

		let url_bytes = words.next()?;
		let url = String::from_utf8(url_bytes.into()).ok()?;

		words.next()?;

		// most browsers send 10-12 headers, and it's not that big of an allocation
		let mut headers = HashMap::with_capacity(12);

		for line in bytes.split(|b| *b == b'\n').skip(1) {
			if line == b"\r" || line.is_empty() {
				break;
			}

			let (key, value) = Self::parse_header(line)?;
			headers.insert(key, value);
		}

		let body = if let Some(position) = bytes.windows(4).position(|window| window == b"\r\n\r\n")
		{
			bytes[position + 4..].into()
		} else {
			vec![]
		};

		Some(Self {
			ip,
			url,
			method,
			body,
			headers,
		})
	}

	/// A function that parses the header form the raw http request headers.
	///
	/// # Arguments
	///
	/// * `line` - It takes a http request header line as a byte slice.
	///
	/// # Returns
	///
	/// It returns an option type of tuple string containing the parsed header key value pairs.
	fn parse_header(line: &[u8]) -> Option<(String, String)> {
		let pos = line.iter().position(|&byte| byte == b':')?;
		let (key, rest) = line.split_at(pos);
		let value = &rest[1..rest.len() - 1];

		Some((
			String::from_utf8_lossy(key).trim().to_string(),
			String::from_utf8_lossy(value).trim().to_string(),
		))
	}

	/// Safely gets a header.
	pub fn get_header(&self, key: &str) -> Option<&str> {
		self.headers.get(key).map(|s| s.as_str())
	}

	/// Equivalent to `get_header(key).unwrap_or(default)`
	pub fn get_header_or(&self, key: &str, default: &'static str) -> &str {
		self.get_header(key).unwrap_or(default)
	}

	/// Checks if a header exists.
	pub fn has_header(&self, key: &str) -> bool {
		self.headers.contains_key(key)
	}

	/// Sets a header using any key and value convertible to Strings
	pub fn set_header<T: ToString, K: ToString>(&mut self, k: T, v: K) {
		self.headers.insert(k.to_string(), v.to_string());
	}

	/// Gets the length of the body.
	pub fn len(&self) -> usize {
		self.body.len()
	}

	/// Checks if the body is empty.
	pub fn is_empty(&self) -> bool {
		self.body.is_empty()
	}

	/// Gets the body as a string.
	/// See [`String::from_utf8_lossy`]
	pub fn text(&self) -> Cow<'_, str> {
		String::from_utf8_lossy(&self.body)
	}

	/// Get the body as a JSON value.
	///
	/// This is only intended for custom invalid JSON handling.
	/// Use [`Request::force_json`] to be able to use the `?` operator.
	#[cfg(feature = "json")]
	pub fn json<T>(&self) -> serde_json::Result<T>
	where
		T: for<'a> serde::de::Deserialize<'a>,
	{
		serde_json::from_slice(&self.body)
	}

	/// Get the body as a JSON value, converting a parse error to a bad request response.
	///
	/// # Example
	/// ```rust
	/// # extern crate serde;
	/// # extern crate serde_json;
	/// use snowboard::Server;
	/// use serde::Deserialize;
	///
	/// #[derive(Deserialize)]
	/// struct MyBody {
	///    foo: String,
	/// }
	///
	/// fn main() -> snowboard::Result {
	///    Server::new("localhost:3000")?.run(|r| {
	///        let body: MyBody = r.force_json()?;
	///
	///        Ok(serde_json::json!({
	///            "foo": body.foo,
	///        }))
	///    })
	/// }
	/// ```
	#[cfg(feature = "json")]
	pub fn force_json<T>(&self) -> Result<T, crate::Response>
	where
		T: for<'a> serde::de::Deserialize<'a>,
	{
		self.json().map_err(|e| e.to_response())
	}

	/// Get a parsed version of the URL.
	/// See [Url]
	pub fn parse_url(&self) -> Url<'_> {
		self.url.as_str().into()
	}

	/// Get the IP address of the client, formatted.
	pub fn pretty_ip(&self) -> String {
		crate::util::format_addr(self.ip)
	}

	/// Check if the request should be kept alive.
	pub fn keep_alive(&self) -> bool {
		self.headers
			.get("connection")
			.map(|s| s.to_ascii_lowercase())
			!= Some("false".to_string())
			|| self
				.headers
				.get("connection")
				.map(|s| s.to_ascii_lowercase())
				== Some("keep-alive".to_string())
	}

	/// Tries to read a request from a stream.
	pub fn read_from<T: io::Write + io::Read>(
		mut stream: T,
		addr: SocketAddr,
		buffer_size: usize,
	) -> io::Result<Request> {
		let mut buffer: Vec<u8> = vec![0; buffer_size];
		let payload_size = stream.read(&mut buffer)?;

		if payload_size == 0 {
			crate::response!(bad_request).send_to(&mut stream)?;
			return Err(io::Error::new(io::ErrorKind::InvalidInput, "Empty request"));
		}

		match Request::new(&buffer[..payload_size], addr) {
			Some(req) => Ok(req),
			None => Err(io::Error::from(io::ErrorKind::InvalidInput)),
		}
	}
}
