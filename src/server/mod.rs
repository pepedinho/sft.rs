use ring::aead::LessSafeKey;
use tokio::net::TcpListener;
pub mod paralelize;

use crate::{
    encryption::Encryption,
    protocol::{Messages, SFT},
    server::paralelize::Parallelizer,
};

pub enum Action {
    Close,
    None,
}

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
    let mut session_key: Option<LessSafeKey> = None;
    loop {
        let msg = match SFT::recv(stream).await {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Client disconnected or error: {e}");
                break;
            }
        };

        match msg {
            Messages::AuthRequest { user, key } => {
                if let Some(shared) = SFT::check_auth(stream, &user, key).await? {
                    session_key = Some(Encryption::derive_key(&shared));
                }
            }
            Messages::Ping => SFT::pong(stream).await?,
            Messages::SessionInit {
                file_count,
                filenames,
            } if session_key.is_some() => {
                let ports = Parallelizer::generate_port(&filenames)?;
                SFT::session_resp(stream, ports.clone()).await?;
                if let Some(shared) = &session_key {
                    Parallelizer::run_workers(file_count, filenames, ports, shared).await?;
                }
            }
            Messages::Close => {
                session_key = None;
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

async fn handle_transfert(stream: &mut tokio::net::TcpStream) -> anyhow::Result<Action> {
    loop {
        let msg = match SFT::recv(stream).await {
            Ok(m) => m,
            Err(_e) => {
                // eprintln!("Client disconnected or error: {e}");
                return Ok(Action::Close);
            }
        };

        

        match msg {
            Messages::FileStart { filename, size } => {
                println!("start file transfert {filename} of size: {size}");
                SFT::send(stream, &Messages::FReady).await?;
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
    Ok(Action::Close)
}
