use snowboard::{headers, response, Server};

#[snowboard::main]
async fn main() -> snowboard::Result {
	let server = Server::new("localhost:8080").await?;

	println!("Listening on {}", server.pretty_addr()?);

	server
		.run(async move |req| {
			println!("{req:#?}");
			response!(ok, "Hello, world!", headers! { "X-Hello" => "World!" })
		})
		.await
}
