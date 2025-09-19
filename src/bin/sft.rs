use std::time::Duration;

use clap::Parser;
use sft::{
    client::cli::{Cli, PackageInfos},
    protocol::SFT,
};
use tokio::{net::TcpStream, time::timeout};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pckginfos = PackageInfos::parse_command(Cli::parse().command)?;
    println!("{:#?}", pckginfos);

    let mut stream = timeout(
        Duration::from_secs(5),
        TcpStream::connect(pckginfos.host + ":8000"),
    )
    .await??;

    let r = SFT::auth(&mut stream).await?;

    println!("{r}");
    SFT::ping(&mut stream).await?;

    SFT::sendf(&mut stream, &pckginfos.file_path).await?;

    SFT::close(&mut stream).await?;
    Ok(())
}
