use snowboard::{Request, ResponseLike, Result, Server};

async fn index(_: Request) -> impl ResponseLike {
	"Async works!"
}

#[smol_potat::main]
async fn main() -> Result {
	Server::new("localhost:8080").await?.run(index).await
}
