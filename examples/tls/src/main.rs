use anyhow::Result;
use tokio::fs;

use snowboard::{Identity, TlsAcceptor};
use snowboard::{Server, SinkExt, StreamExt, WebSocket};

async fn echo(ws: WebSocket) {
	let (mut sender, mut reciever) = ws.split();
	while let Some(Ok(msg)) = reciever.next().await {
		let _ = sender.send(msg).await;
	}
}

#[tokio::main]
async fn main() -> Result<()> {
	let password = "1234";
	let der = fs::read("identity.pfx").await?;
	let identity = Identity::from_pkcs12(&der, password)?;
	let tls_acceptor = TlsAcceptor::new(identity)?;

	Ok(Server::new("localhost:3000", tls_acceptor)?
		.on_websocket("/ws", echo)
		.run(async move |request| format!("{request:#?}"))
		.await?)
}
