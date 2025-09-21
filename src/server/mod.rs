use tokio::net::TcpListener;
pub mod paralelize;

use crate::{
    protocol::{Messages, SFT},
    server::paralelize::Parallelizer,
};

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
    loop {
        let msg = match SFT::recv(stream).await {
            Ok(m) => m,
            Err(_e) => {
                // eprintln!("Client disconnected or error: {e}");
                break;
            }
        };

        match msg {
            Messages::AuthRequest { user, key } => SFT::check_auth(stream, &user, &key).await?,
            Messages::Ping => SFT::pong(stream).await?,
            Messages::SessionInit {
                file_count,
                filenames,
            } => {
                let ports = Parallelizer::generate_port(&filenames)?;
                SFT::session_resp(stream, ports.clone()).await?;
                Parallelizer::run_workers(file_count, filenames, ports).await?;
            }
            Messages::Close => {
                println!("Client requested close");
                break;
            }
            _ => {
                SFT::send(
                    stream,
                    &Messages::Error {
                        msg: "Unknown request".into(),
                    },
                )
                .await?;
                println!("unknown request {:#?}", msg);
                break;
            }
        }
    }
    Ok(())
}

async fn handle_transfert(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
    loop {
        let msg = match SFT::recv(stream).await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Client disconnected or error: {e}");
                break;
            }
        };

        match msg {
            Messages::FileStart { filename, size } => {
                println!("start file transfert {filename} of size: {size}");
                SFT::recvf(stream, &filename).await?;
            }
            Messages::Close => {
                println!("Client requested close");
                break;
            }
            _ => {
                SFT::send(
                    stream,
                    &Messages::Error {
                        msg: "Unknown request".into(),
                    },
                )
                .await?;
                println!("unknown request {:#?}", msg);
                break;
            }
        }
    }
    Ok(())
}
