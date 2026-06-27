use snowboard::{Result, Server, TcpListener};

#[tokio::main]
async fn main() -> Result {
	let server = Server::new("localhost:8080")?;
	let listener = TcpListener::bind(server.addr()).await?;
	match server.try_accept(&listener).await {
		Ok((_, ip)) => println!("received connection from {}", ip),
		Err(_) => println!("a connection has failed !"),
	}

	Ok(())
}
