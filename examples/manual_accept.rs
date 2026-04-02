use snowboard::{Result, Server};

#[smol_potat::main]
async fn main() -> Result {
	let server = Server::new("localhost:8080").await?;
	match server.try_accept().await {
		Ok((_, ip)) => println!("recieved connection from {}", ip),
		Err(_) => println!("a connection has failed !"),
	}

	Ok(())
}
