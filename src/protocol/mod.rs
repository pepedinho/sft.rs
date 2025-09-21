use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Messages is a enum that reference all messages type sft protocol can process
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Messages {
    AuthRequest { user: String, key: String },
    AuthResponse { ok: bool, msg: String },
    FileStart { filename: String, size: u64 },
    FileChunk { data: Vec<u8> },
    FileEnd,
    Progress { byte_received: u64, total_byte: u64 },
    Error { msg: String },
    Ack, // Acknowledgment of receipt
    Close,
    Ping,
    Pong,
}

pub struct SFT {}

impl SFT {
    // -----------------------
    // Auth methods
    // -----------------------

    pub async fn auth(stream: &mut tokio::net::TcpStream) -> anyhow::Result<String> {
        SFT::send(
            stream,
            &Messages::AuthRequest {
                user: "pepe".to_string(),
                key: "oui".to_string(),
            },
        )
        .await?;
        let msg = SFT::recv(stream).await?;
        println!("debug: recv msg:\n {:#?}", msg);
        match msg {
            Messages::AuthResponse { ok: false, msg: m } => Err(anyhow::anyhow!(m)),
            Messages::AuthResponse { ok: true, msg: m } => Ok(m),
            _ => Err(anyhow::anyhow!("Internal server error")),
        }
    }

    pub async fn check_auth(
        stream: &mut tokio::net::TcpStream,
        user: &str,
        key: &str,
    ) -> anyhow::Result<()> {
        if user.is_empty() || key.is_empty() {
            SFT::send(
                stream,
                &Messages::AuthResponse {
                    ok: false,
                    msg: "auth failed invalid credential".to_string(),
                },
            )
            .await?;
            return Ok(());
        }
        SFT::send(
            stream,
            &Messages::AuthResponse {
                ok: true,
                msg: "connexion establish !".to_string(),
            },
        )
        .await?;
        Ok(())
    }

    // -----------------------
    // I/O methods
    // -----------------------

    pub async fn recv(stream: &mut tokio::net::TcpStream) -> anyhow::Result<Messages> {
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let lenght = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; lenght];
        stream.read_exact(&mut buf).await?;

        let msg: Messages = serde_json::from_slice(&buf)?;
        Ok(msg)
    }

    pub async fn send(stream: &mut tokio::net::TcpStream, msg: &Messages) -> anyhow::Result<()> {
        let payload = serde_json::to_vec(msg)?;
        let len = (payload.len() as u32).to_be_bytes();

        stream.write_all(&len).await?;
        stream.write_all(&payload).await?;
        Ok(())
    }

    pub async fn sendf(stream: &mut tokio::net::TcpStream, path: &str) -> anyhow::Result<()> {
        let file = tokio::fs::File::open(path).await?;
        let metadata = file.metadata().await?;
        let size = metadata.len();

        let filename = std::path::Path::new(path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        println!("debug: send file start");

        SFT::send(stream, &Messages::FileStart { filename, size }).await?;

        println!("debug: start file streaming");

        stream_file_content(file, stream, size).await?;
        SFT::send(stream, &Messages::FileEnd).await?;
        loop {
            let resp = SFT::recv(stream).await?;
            match resp {
                Messages::Ack => {
                    println!("File upload with success !");
                    break;
                }
                Messages::Error { msg } => anyhow::bail!("Internal server error: {msg}"),
                Messages::Progress {
                    byte_received,
                    total_byte,
                } => println!("progression : {byte_received} => {total_byte}"),
                _ => {
                    println!("debug: server response : {:#?}", resp);
                    anyhow::bail!("Unexpected Response");
                }
            }
        }
        Ok(())
    }

    // TODO: send Progress only every 5/6 Mb to synch with client progress (need to increase speed)
    pub async fn recvf(stream: &mut tokio::net::TcpStream, filename: &str) -> anyhow::Result<()> {
        let path = format!("uploads/{filename}");
        let mut file = tokio::fs::File::create(path).await?;

        loop {
            match SFT::recv(stream).await? {
                Messages::FileChunk { data } => {
                    file.write_all(&data).await?;
                }
                Messages::FileEnd => {
                    drop(file);
                    println!("debug: recv EOF");
                    SFT::send(stream, &Messages::Ack).await?;
                    break;
                }
                _ => {
                    println!("debug: recv unexpected signal");
                    SFT::send(
                        stream,
                        &Messages::Error {
                            msg: "Unexpected message".into(),
                        },
                    )
                    .await?;
                    println!("error: unexpected messages");
                    break;
                }
            }
        }
        Ok(())
    }

    pub async fn ping(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("ping");
        SFT::send(stream, &Messages::Ping).await?;
        match SFT::recv(stream).await? {
            Messages::Pong => Ok(()),
            _ => Err(anyhow::anyhow!("unexpected response")),
        }
    }

    pub async fn pong(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("pong");
        SFT::send(stream, &Messages::Pong).await?;
        Ok(())
    }

    pub async fn close(stream: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
        println!("debug: send close order");
        SFT::send(stream, &Messages::Close).await?;
        Ok(())
    }
}

async fn stream_file_content(
    mut file: tokio::fs::File,
    stream: &mut tokio::net::TcpStream,
    size: u64,
) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 64 * 2048];
    let mut total_send = 0u64;
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        total_send += n as u64;

        SFT::send(
            stream,
            &Messages::FileChunk {
                data: buf[..n].to_vec(),
            },
        )
        .await?;

        println!("Progress: {total_send} => {size}");

        // if let Messages::Progress {
        //     byte_received,
        //     total_byte,
        // } = SFT::recv(stream).await?
        // {
        //     println!("Progress: {}/{}", byte_received, total_byte);
        // }
    }
    Ok(())
}
