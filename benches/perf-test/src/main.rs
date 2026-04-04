#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use snowboard::{Result, Server};

#[tokio::main]
async fn main() -> Result {
	// Not returning anything (`()`) is the same as Response::default()
	Server::new("localhost:8080")?
		.with_buffer_size(200)
		.run(async |_| {})
		.await
}
