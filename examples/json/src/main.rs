use serde_json::Value;
use snowboard::{Response, Server};

#[derive(serde::Deserialize)]
struct Example {
	number: isize,
}

#[snowboard::main]
async fn main() -> snowboard::Result {
	Server::new("localhost:8080")
		.await?
		.run(async |req| -> Result<Value, Response> {
			let parsed: Example = req.expect_json()?;

			Ok(serde_json::json!({
				"number_plus_one": parsed.number + 1
			}))
		})
		.await
}
