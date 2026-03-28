use anyhow::Result;
use snowboard::TlsAcceptor;

use snowboard::Server;

use smol::fs::File;

#[smol_potat::main]
async fn main() -> Result<()> {
	let password = "1234";
	let idx = File::open("identity.pfx").await?;
	let tls_acceptor = TlsAcceptor::new(idx, password).await?;

	Server::new("localhost:3000", tls_acceptor)
		.await?
		.run(async move |request| format!("{request:#?}"))
		.await
}
