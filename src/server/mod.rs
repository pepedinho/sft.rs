use tokio::net::TcpListener;

use crate::protocol::{Messages, SFT};

pub struct Listener {}

impl Listener {
    pub async fn listen_on(addr: &str) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        println!("listening started, ready to accept");

        loop {
            let (mut socket, addr) = listener.accept().await?;
            println!("new connexion from {:?}", addr);

            tokio::spawn(async move {
                if let Err(e) = handle_client(&mut socket).await {
                    eprintln!("Error with client {:?}: {:?}", addr, e);
                }
            });
        }
    }
}

async fn handle_client(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
    let msg = SFT::recv(stream).await?;

    match msg {
        Messages::AuthRequest { user, key } => SFT::check_auth(stream, &user, &key).await?,
        _ => {
            println!("unknown request")
        }
    }
    Ok(())
}
