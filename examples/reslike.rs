use snowboard::{Response, ResponseLike, Server};

struct Example {
	num: usize,
}

impl ResponseLike for Example {
	fn to_response(self) -> Response {
		snowboard::response!(ok, format!("My favorite number is {}!", self.num))
	}
}

#[snowboard::main]
async fn main() -> snowboard::Result {
	Server::new("localhost:8080")
		.await?
		.run(async |_| Example { num: 5 })
		.await;
}
