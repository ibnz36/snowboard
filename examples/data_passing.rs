use snowboard::{response, Result, Server};
use std::sync::Arc;

struct ServerData {
	hello: String,
}

#[smol_potat::main]
async fn main() -> Result {
	let data = Arc::new(ServerData {
		hello: "hi!".into(),
	});

	Server::new("localhost:8080")?
		.run(move |_| {
			let data = Arc::clone(&data);

			async move { response!(ok, data.hello.clone()) }
		})
		.await
}
