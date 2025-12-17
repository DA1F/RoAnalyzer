use ro_grpc::DeviceGrpcClient;
use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let endpoint = args.get(1).map(|s| s.as_str()).unwrap_or("http://127.0.0.1:50051");

    match DeviceGrpcClient::connect(endpoint.to_string()).await {
        Ok(mut client) => {
            println!("Connected to {}", endpoint);
            match client.get_clipboard().await {
                Ok(text) => println!("Clipboard: {}", text),
                Err(e) => eprintln!("GetClipboard failed: {}", e),
            }

            // Demonstration: perform a tap at 100,200
            match client.tap(100, 200).await {
                Ok(()) => println!("Tap sent: (100,200)"),
                Err(e) => eprintln!("Tap failed: {}", e),
            }
        }
        Err(e) => eprintln!("Failed to connect: {}", e),
    }
}
