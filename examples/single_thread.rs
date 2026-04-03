use snowboard::{response, Request, Result, Server, TcpListener};

#[smol_potat::main]
async fn main() -> Result {
	let mut server = Server::new("localhost:8080")?;
	let listener = TcpListener::bind(server.addr()).await?;

	loop {
		let (mut stream, ip) = server.next_stream(&listener).await;
		let Ok(req) = Request::read_from(&mut stream, ip, 1000).await else {
			continue;
		};

		println!("request from {}: {:#?}", ip, req);

		let _ = response!(ok, "Hello, world!").send_to(&mut stream).await;
	}
}
