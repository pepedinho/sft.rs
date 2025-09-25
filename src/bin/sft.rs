use std::time::Duration;

use clap::Parser;
use ring::aead::LessSafeKey;
use sft::{
    client::{
        channel::Channel,
        cli::{Cli, PackageInfos},
    },
    encryption::Encryption,
    protocol::SFT,
};
use tokio::{net::TcpStream, time::timeout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pckginfos = PackageInfos::parse_command(Cli::parse().command)?;
    println!("{:#?}", pckginfos);

    let host = pckginfos.host.clone();

    let mut stream = timeout(Duration::from_secs(5), TcpStream::connect(host + ":8000")).await??;

    let r = SFT::auth(&mut stream).await?;
    let unbound = Encryption::derive_key(&r);
    let session_key = LessSafeKey::new(unbound);

    // println!("{r}");
    SFT::ping(&mut stream).await?;

    Channel::channelised_send(&mut stream, pckginfos, &session_key).await?;

    // SFT::sendf(&mut stream, &pckginfos.file_path[0]).await?;

    // SFT::close(&mut stream).await?;
    Ok(())
}
