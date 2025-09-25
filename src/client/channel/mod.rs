use indicatif::MultiProgress;
use ring::aead::LessSafeKey;
use tokio::net::TcpStream;
use x25519_dalek::SharedSecret;

use crate::{client::cli::PackageInfos, protocol::SFT};

pub struct Channel {}

impl Channel {
    pub async fn channelised_send(
        stream: &mut tokio::net::TcpStream,
        pckinfo: PackageInfos,
        shared_key: &LessSafeKey,
    ) -> anyhow::Result<()> {
        SFT::ping(stream).await?;

        let files = pckinfo.file_path.clone();
        let ports = SFT::open_session(stream, &files).await?;

        let mut handles = Vec::new();
        let mp = MultiProgress::new();

        for (i, f) in files.iter().enumerate() {
            let file = f.clone();

            // println!("client: treat {file} on port {}", ports[i]);
            let addr = pckinfo.host.to_owned() + ":" + &ports[i].to_string();
            let pb_mp = mp.clone();
            let handle = tokio::spawn(async move {
                // println!("client: worker run");
                let mut s = match TcpStream::connect(addr.clone()).await {
                    Err(e) => {
                        println!("Error: failed to connect on {addr}: {e}");
                        return;
                    }
                    Ok(s) => s,
                };

                // println!("client worker: tcp connexion success on {addr}");

                if let Err(e) = SFT::sendf(&mut s, &file, &pb_mp).await {
                    println!("Error: failed to connect on {addr}: {e}");
                    return;
                };
                if let Err(e) = SFT::close(&mut s).await {
                    println!("Error: failed to connect on {addr}: {e}");
                    return;
                };
            });
            handles.push(handle);
        }

        for h in handles {
            h.await?;
        }

        SFT::close(stream).await?;

        Ok(())
    }
}
