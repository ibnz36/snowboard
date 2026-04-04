use snowboard::{headers, response, Server};

#[smol_potat::main]
async fn main() -> snowboard::Result {
	let server = Server::new("localhost:8080")?;

	println!("Listening on {}", server.pretty_addr());

	server
		.run(async move |req| {
			println!("{req:#?}");
			response!(ok, "Hello, world!", headers! { "X-Hello" => "World!" })
		})
		.await
}
