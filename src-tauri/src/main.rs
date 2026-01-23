// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "rzem-ai-inference")]
#[command(about = "Local AI Image Generation with GPU Sharing", long_about = None)]
struct Args {
    /// Enable server mode (expose REST/WebSocket API)
    #[arg(long)]
    server: bool,

    /// Run in headless mode (no desktop UI, server mode only)
    #[arg(long)]
    headless: bool,

    /// Connect to remote server (client mode)
    #[arg(long)]
    client: bool,

    /// Server URL for client mode (e.g., http://192.168.1.100:8080)
    #[arg(long, value_name = "URL")]
    server_url: Option<String>,

    /// Port for server mode (default: 8080)
    #[arg(long, short = 'p', default_value = "8080")]
    port: u16,
}

fn main() {
    let args = Args::parse();

    // Determine operation mode
    let (runtime_config, port) = if args.client {
        // Client mode
        let server_url = args.server_url
            .expect("--server-url is required when using --client");

        (rzem_ai_inference::shared::protocol::RuntimeConfig::client(server_url), None)
    } else if args.server {
        // Server mode
        if args.headless {
            eprintln!("WARNING: Headless mode not yet implemented. Running with desktop UI.");
        }

        println!("Starting in server mode on port {}...", args.port);
        println!("API will be available at: http://localhost:{}/api/v1", args.port);
        println!("WebSocket at: ws://localhost:{}/api/v1/ws", args.port);
        println!("\nWARNING: No authentication enabled. Use on trusted networks only!");

        (rzem_ai_inference::shared::protocol::RuntimeConfig::server(args.port), Some(args.port))
    } else {
        // Local mode (default)
        (rzem_ai_inference::shared::protocol::RuntimeConfig::local(), None)
    };

    rzem_ai_inference::run_with_config(runtime_config, port)
}
