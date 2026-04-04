macro_rules! map_into {
	($($name:expr_2021 => $val:expr_2021 $(,)?)*) => {
		{
			let mut map = HashMap::new();
			$(map.insert($name.into(), $val.into());)*
			map
		}
	};
}

use std::collections::HashMap;

use snowboard::{Method, Request, Url};

#[test]
fn parse_request() {
	let request = b"HEAD / HTTP/1.1\r\nHost: localhost:8080\r\nUser-Agent: curl/xx\r\nAccept: */*\r\n\r\nBODY, BODY, BODY\nMORE BODY\n";

	let sample_ip = "127.0.0.1:8080".parse().unwrap();

	assert_eq!(
		Request::new(request, sample_ip).unwrap(),
		Request {
			ip: sample_ip,
			url: "/".into(),
			method: Method::HEAD,
			body: "BODY, BODY, BODY\nMORE BODY\n".into(),
			headers: map_into! {
				"Host" => "localhost:8080",
				"User-Agent" => "curl/xx",
				"Accept" => "*/*",
			}
		}
	);
}

#[test]
fn parse_invalid_utf8() {
	let mut request = b"GET / HTTP/1.1\r\nX-A: B\r\n\r\n".to_vec();

	// Invalid UTF-8 bytes
	request.push(0x80);
	request.push(0xFF);
	request.push(0xC0);

	let sample_ip = "127.0.0.1:8080".parse().unwrap();

	let parsed = Request::new(&request, sample_ip).unwrap();

	assert_eq!(
		parsed,
		Request {
			ip: sample_ip,
			url: "/".into(),
			method: Method::GET,
			body: vec![0x80, 0xFF, 0xC0],
			headers: map_into! {
				"X-A" => "B",
			}
		}
	);

	// Invalid UTF-8 bytes get converted to the replacement character (�)
	assert_eq!(parsed.text(), "���")
}

#[test]
fn test_different_amount_of_headers() {
	let sample_ip = "127.0.0.1:8080".parse().unwrap();

	let base_request = b"GET / HTTP/1.1\r\nHost: localhost:8080\r\n";

	for i in 0..20 {
		let mut request = base_request.to_vec();

		for _ in 0..i {
			request.extend_from_slice(b"A: B\r\n");
		}

		request.extend_from_slice(b"\r\n");
		request.extend_from_slice(b"h");

		let parsed = Request::new(&request, sample_ip).unwrap();

		let mut headers = HashMap::new();
		for _ in 0..i {
			headers.insert("A".into(), "B".into());
		}

		headers.insert("Host".into(), "localhost:8080".into());

		assert_eq!(
			parsed,
			Request {
				ip: sample_ip,
				url: "/".into(),
				method: Method::GET,
				body: b"h".into(),
				headers
			}
		);
	}
}

#[test]
fn parse_url() {
	let complex = "/path/to/something?param1=value1&param2=value2&param3=value3&s=&";
	let simple = "/a/b?c=d";
	let base = "/";
	let weird = "/?&=";
	let no_query = "/a/b/c";

	assert_eq!(
		Url::from(complex),
		Url {
			path: vec!["path", "to", "something"],
			search_params: map_into! {
				"param1" => "value1",
				"param2" => "value2",
				"param3" => "value3",
				"s" => "",
			}
		}
	);

	assert_eq!(
		Url::from(simple),
		Url {
			path: vec!["a", "b"],
			search_params: map_into! {
				"c" => "d",
			}
		}
	);

	assert_eq!(
		Url::from(base),
		Url {
			path: vec![],
			search_params: HashMap::new(),
		}
	);

	assert_eq!(
		Url::from(weird),
		Url {
			path: vec![],
			search_params: HashMap::new()
		}
	);

	assert_eq!(
		Url::from(no_query),
		Url {
			path: vec!["a", "b", "c"],
			search_params: HashMap::new()
		}
	);
}
