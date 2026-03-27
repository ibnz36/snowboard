use snowboard::{Request, ResponseLike, Result, Server};

async fn index(_: Request) -> impl ResponseLike {
	"Async works!"
}

fn main() -> Result {
	Server::new("localhost:8080")?.run(index)
}
