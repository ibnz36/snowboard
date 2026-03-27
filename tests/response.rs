use snowboard::{headers, response, Response};

#[test]
fn response_generation() {
	let default_res = Response::default();

	assert_eq!(default_res, response!(ok));
	assert_eq!(default_res.to_string(), "HTTP/1.1 200 Ok\r\n\r\n");

	let custom = response!(not_found, "Custom body");

	assert_eq!(
		custom.to_string(),
		"HTTP/1.1 404 Not Found\r\n\r\nCustom body"
	);

	let with_headers = response!(
		ok,
		[], // No body
		headers! {
			"Content-Type" => "text/html",
			"X-My-Header" => 1234,
		}
	)
	.to_string();

	assert!(with_headers.contains("Content-Type: text/html"));
	assert!(with_headers.contains("X-My-Header: 1234"));
	assert!(with_headers.contains("HTTP/1.1 200 Ok"));

	let custom_http_version = response!(
		switching_protocols,
		[],          // No body
		headers! {}  // No headers
	);

	assert_eq!(
		custom_http_version.to_string(),
		"HTTP/1.1 101 Switching Protocols\r\n\r\n"
	);
}
