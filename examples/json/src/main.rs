use serde_json::Value;
use snowboard::{Response, Server};

#[derive(serde::Deserialize)]
struct Example {
	number: isize,
}

#[smol_potat::main]
async fn main() -> snowboard::Result {
	Server::new("localhost:8080")
		.await?
		.run(async |req| -> Result<Value, Response> {
			let example: Example = req.expect_json()?;

			Ok(serde_json::json!({
				"number_plus_one": example.number + 1
			}))
		})
		.await
}
