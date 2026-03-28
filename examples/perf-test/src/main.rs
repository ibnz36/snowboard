#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use snowboard::{Result, Server};

#[smol_potat::main(threads = 12)]
async fn main() -> Result {
	// Not returning anything (`()`) is the same as Response::default()
	Server::new("localhost:8080")
		.await?
		.with_buffer_size(200)
		.run(async |_| {})
		.await;
}
