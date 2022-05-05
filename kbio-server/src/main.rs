// Since we can hardly control every thread, we should use `taskset`.

use config::{ServerConfig, PACKET_SIZE};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    runtime::{Builder},
};

fn main() {
    let cfg = ServerConfig::parse();
    let cores = cfg.cores.len();
    println!(
        "Running ping pong server with Tokio.\nPacket size: {}\nListen {}\nCPU count: {}",
        PACKET_SIZE, cfg.bind, cores
    );
    let rt = Builder::new_multi_thread()
        .enable_all()
        .worker_threads(cores)
        .uring_entries(32768)
        .build()
        .unwrap();
    rt.block_on(async move {
        tokio::spawn(serve(cfg)).await.unwrap();
    })
}

async fn serve(cfg: ServerConfig) {
    let listener = TcpListener::bind(&cfg.bind).await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            let mut buf = vec![0; PACKET_SIZE];
            loop {
                match stream.read_exact(&mut buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }
                match stream.write_all(&buf).await {
                    Ok(_) => {}
                    Err(_) => {
                        return;
                    }
                }
            }
        });
    }
}
