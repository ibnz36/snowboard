//! A module that provides and handles traits which can help in serializing and deserializing
//! response into different data types.

use super::Response;

/// A trait for everything that can be converted into a Response.
pub trait ResponseLike {
	/// Converts `self` into a `Response`.
	fn to_response(self) -> Response;
}

impl ResponseLike for Response {
	#[inline]
	fn to_response(self) -> Response {
		self
	}
}

impl ResponseLike for () {
	#[inline]
	fn to_response(self) -> Response {
		Response::default()
	}
}

impl ResponseLike for &str {
	#[inline]
	fn to_response(self) -> Response {
		crate::response!(
			ok,
			self,
			crate::headers! { "Content-Type" => "text/plain; charset=utf-8" }
		)
	}
}

impl ResponseLike for String {
	#[inline]
	fn to_response(self) -> Response {
		crate::response!(
			ok,
			self,
			crate::headers! { "Content-Type" => "text/plain; charset=utf-8" }
		)
	}
}

impl ResponseLike for Vec<u8> {
	#[inline]
	fn to_response(self) -> Response {
		crate::response!(ok, self)
	}
}

// Particuraly useful for `?` operators when using outside functions.
impl<T, E> ResponseLike for Result<T, E>
where
	T: ResponseLike,
	E: ResponseLike,
{
	fn to_response(self) -> Response {
		match self {
			Ok(res) => res.to_response(),
			Err(res) => res.to_response(),
		}
	}
}

#[cfg(feature = "json")]
impl ResponseLike for serde_json::Error {
	#[inline]
	fn to_response(self) -> Response {
		let bytes = self.to_string().into_bytes();

		crate::response!(
			bad_request,
			bytes,
			crate::headers! {
				"Content-Type" => "text/plain; charset=utf-8",
			}
		)
	}
}

#[cfg(feature = "json")]
impl ResponseLike for serde_json::Value {
	#[inline]
	fn to_response(self) -> Response {
		let bytes = serde_json::to_vec(&self).unwrap_or_else(|_| self.to_string().into_bytes());

		crate::response!(
			ok,
			bytes,
			crate::headers! {
				"Content-Type" => "application/json; charset=utf-8",
			}
		)
	}
}
