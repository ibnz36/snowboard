use snowboard::{response, Request, Result, Server};

fn main() -> Result {
	let server = Server::new("localhost:8080")?;

	for (mut stream, ip) in server {
		let Ok(req) = Request::read_from(&mut stream, ip, 1000) else {
			continue;
		};

		println!("request from {}: {:#?}", ip, req);

		let _ = response!(ok, "Hello, world!").send_to(&mut stream);
	}

	Ok(())
}
