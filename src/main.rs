use std::{
    net::SocketAddr,
    path::PathBuf,
};

use mediagetter::{application, Config};

#[tokio::main]
async fn main() {
    let config = Config {
        addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        dir: PathBuf::from("test"),
    };
    application(config).await.unwrap();
}
