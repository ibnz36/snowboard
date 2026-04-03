use snowboard::Server;
use snowboard::{StreamExt, WebSocket};

async fn handle_ws(ws: WebSocket) {
	let (mut sender, mut reciever) = ws.split();
	while let Some(Ok(msg)) = reciever.next().await {
		let _ = sender.send(msg).await;
	}
}

#[smol_potat::main]
async fn main() -> snowboard::Result {
	Server::new("localhost:8080")?
		.on_websocket("/ws", handle_ws)
		.run(async |_| "Try `/ws`!")
		.await
}
