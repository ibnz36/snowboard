//! A module that provides code and other modules to serialize/deserialize response into appropriate
//! data types.

mod response_types;
mod responselike;

pub use responselike::ResponseLike;
use smol::io::{AsyncWrite, AsyncWriteExt};

use std::{collections::HashMap, fmt, fmt::Write, io};

use crate::HttpVersion;

/// The default HTTP version used by the server.
pub const DEFAULT_HTTP_VERSION: HttpVersion = HttpVersion::V1_1;

/// Response struct.
/// Contains the response data and converts it to text if needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
	/// HTTP protocol version.
	/// Do note the server only supports HTTP/1.1, so even if
	/// this is specified as HTTP/2.0 or any other, it'll still
	/// be a HTTP/1.1 response.
	pub(crate) version: HttpVersion,
	/// HTTP status code.
	pub status: u16,
	/// According text for the status.
	pub status_text: &'static str,
	/// The request body, stored in bytes.
	pub bytes: Vec<u8>,
	/// Headers of the response
	pub headers: Headers,
}

/// Equivalent to `HashMap<&'static str, String>`.
pub type Headers = HashMap<&'static str, String>;

impl Response {
	/// Manually create a Response instance.
	/// Also see the [`crate::response!`] macro, which is the recommended way to create responses.
	pub fn new(
		version: HttpVersion,
		status: u16,
		status_text: &'static str,
		bytes: Vec<u8>,
		headers: Headers,
	) -> Self {
		Self {
			version,
			status,
			status_text,
			bytes,
			headers,
		}
	}

	/// Writes the response, to a stream.
	pub async fn send_to<T: AsyncWrite + Unpin>(
		&mut self,
		stream: &mut T,
	) -> Result<(), io::Error> {
		self.set_content_length();
		let prev = self.prepare_response().into_bytes();
		stream.write_all(&prev).await?;
		stream.write_all(&self.bytes).await?;
		stream.write_all(b"\r\n").await?;
		stream.flush().await
	}

	/// Sets a header to the response, returning the response itself.
	/// Use Response::with_content_type for the 'Content-Type' header.
	pub fn with_header(mut self, key: &'static str, value: String) -> Self {
		self.headers.insert(key, value);
		self
	}

	/// Sets the content type of the response, returning the response itself.
	/// Note that this does not check if the content type is valid, so be careful.
	pub fn with_content_type(self, value: String) -> Self {
		self.with_header("Content-Type", value)
	}

	/// Sets the content length of a reference to a response
	pub fn set_header(&mut self, key: &'static str, value: String) -> &mut Self {
		self.headers.insert(key, value);

		self
	}

	/// Sets the content length of the response
	pub fn set_content_length(&mut self) -> &mut Self {
		// the +2 is because of the obligatory "\r\n" at the end of responses
		self.set_header("Content-Length", (self.len() + 2).to_string())
	}

	/// Returns the first lines of the generated response. (everything except the body)
	/// This function is used internally to create the response.
	fn prepare_response(&self) -> String {
		let estimated_size = 50 + (self.headers.len() * 100) + 2;
		let mut text = String::with_capacity(estimated_size);

		writeln!(
			text,
			"{} {} {}",
			self.version, self.status, self.status_text
		)
		.unwrap();

		for (key, value) in self.headers.iter() {
			writeln!(text, "{}: {}", key, value).unwrap();
		}

		text.push_str("\r\n");
		text
	}

	/// Converts the `Response` into a HTTP Response, as bytes.
	pub fn to_bytes(&mut self) -> Vec<u8> {
		let mut bytes = self.prepare_response().into_bytes();
		bytes.append(&mut self.bytes);
		bytes
	}

	/// Gets the length of the response body.
	pub fn len(&self) -> usize {
		self.bytes.len()
	}

	/// Checks if the response body is empty.
	pub fn is_empty(&self) -> bool {
		self.bytes.is_empty()
	}

	/// Adds optional but useful headers to a response.
	/// This includes the Content-Length header, Date header and Server header.
	pub fn with_default_headers(mut self) -> Self {
		let now = chrono::Utc::now().to_rfc2822();
		let len = self.len();

		self.set_header("Content-Length", len.to_string())
			.set_header("Date", now)
			.set_header("Server", "Snowboard".into());

		self
	}

	/// Used internally to add default headers if needed.
	pub(crate) fn maybe_add_defaults(mut self, should_insert: bool) -> Self {
		if should_insert {
			self = self.with_default_headers();
		}

		self
	}
}

impl From<Response> for Vec<u8> {
	fn from(mut res: Response) -> Self {
		res.to_bytes()
	}
}

impl fmt::Display for Response {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut text = self.prepare_response();
		text += String::from_utf8_lossy(&self.bytes).as_ref();
		text += "\r\n";
		write!(f, "{}", text)
	}
}

impl Default for Response {
	fn default() -> Self {
		Self {
			version: DEFAULT_HTTP_VERSION,
			status: 200,
			status_text: "OK",
			bytes: vec![],
			headers: Headers::default(),
		}
	}
}
