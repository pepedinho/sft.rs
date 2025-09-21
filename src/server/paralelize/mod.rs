use tokio::net::TcpListener;

use crate::server::handle_transfert;

pub struct Parallelizer {}

const DEFAULT_ADDRES: &str = "0.0.0.0:";

fn get_free_port() -> anyhow::Result<u16> {
    let listener = std::net::TcpListener::bind(DEFAULT_ADDRES.to_owned() + "0")?;
    let addr: std::net::SocketAddr = listener.local_addr()?;
    let port = addr.port();

    Ok(port)
}

impl Parallelizer {
    pub fn generate_port(filenames: &Vec<String>) -> anyhow::Result<Vec<u16>> {
        let mut ports = Vec::new();
        for _ in filenames {
            let port = get_free_port()?;
            ports.push(port);
        }
        Ok(ports)
    }
    pub async fn run_workers(
        file_count: usize,
        filenames: Vec<String>,
        ports: Vec<u16>,
    ) -> anyhow::Result<()> {
        if file_count != filenames.len() {
            anyhow::bail!("invalid request");
        }

        let mut handles = Vec::new();

        for (i, file) in filenames.iter().enumerate() {
            let addr = DEFAULT_ADDRES.to_owned() + &ports[i].to_string();

            let f = file.clone();
            let handle = tokio::spawn(async move {
                let listener = TcpListener::bind(addr.clone())
                    .await
                    .expect("failed to bind listener");
                println!("listening for file {f} on {addr}");

                loop {
                    let (mut socket, addr) = match listener.accept().await {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Accept error on {addr}: {:?}", e);
                            continue;
                        }
                    };

                    tokio::spawn(async move {
                        println!("New file transfert");
                        if let Err(e) = handle_transfert(&mut socket).await {
                            eprintln!("Error with client {:?}: {:?}", addr, e);
                        }
                    });
                }
            });

            handles.push(handle);
            // Listener::listen_transfert(&addr).await?;
        }

        for h in handles {
            h.await?;
        }

        todo!()
    }
}
