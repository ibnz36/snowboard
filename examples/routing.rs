use snowboard::{response, Request, ResponseLike, Result, Server};

async fn router(req: Request) -> impl ResponseLike {
	// /{x}
	match req.parse_url().at(0) {
		Some("ping") => response!(ok, "Pong!"),
		Some("api") => response!(continue),
		None => response!(ok, "Hello, world!"),
		_ => response!(not_found, "Route not found"),
	}
}

#[smol_potat::main]
async fn main() -> Result {
	Server::new("localhost:8080")?.run(router).await
}
