use std::{
    net::SocketAddr,
    path::PathBuf,
};

use mediagetter::{application, Config};

#[tokio::main]
async fn main() {
    let config = Config {
        addr: SocketAddr::from(([0, 0, 0, 0], 6969)),
        dir: PathBuf::from("/home/emby/art"),
    };
    application(config).await.unwrap();
}
