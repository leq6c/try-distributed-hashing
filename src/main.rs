use std::env;

mod server;
mod client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} server|client", args[0]);
        return Ok(());
    }

    match args[1].as_str() {
        "server" => {
            let port = args.get(2).expect("Expected port number as the second argument");
            let addr = format!("0.0.0.0:{}", port);
            print!("Starting server on: {}", addr);
            server::run(addr).await?;
        }
        "client" => client::run_client().await?,
        _ => {
            eprintln!("Usage: {} server|client", args[0]);
        }
    }

    Ok(())
}
