use snowboard::Server;
use snowboard::WebSocket;

fn handle_ws(mut ws: WebSocket) {
	while let Ok(msg) = ws.read() {
		let _ = ws.send(msg);
	}
}

fn main() -> snowboard::Result {
	Server::new("localhost:3000")?
		.on_websocket("/ws", handle_ws)
		.run(async |_| "Try `/ws`!")
}
