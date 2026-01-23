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

    // Check environment variables for dev mode support
    // These allow Tauri dev mode to work since CLI args can't be passed easily
    let client_mode = args.client || std::env::var("RZEM_CLIENT_MODE").is_ok();
    let server_mode = args.server || std::env::var("RZEM_SERVER_MODE").is_ok();
    let server_url_env = std::env::var("RZEM_SERVER_URL").ok();
    let port_env = std::env::var("RZEM_PORT").ok()
        .and_then(|p| p.parse::<u16>().ok());

    // Determine operation mode
    let (runtime_config, port) = if client_mode {
        // Client mode - prefer CLI arg, fallback to env var
        let server_url = args.server_url
            .or(server_url_env)
            .expect("--server-url or RZEM_SERVER_URL is required when using --client or RZEM_CLIENT_MODE");

        println!("Starting in client mode...");
        println!("Connecting to: {}", server_url);

        (rzem_ai_inference::shared::protocol::RuntimeConfig::client(server_url), None)
    } else if server_mode {
        // Server mode - prefer CLI arg, fallback to env var
        let port = port_env.unwrap_or(args.port);

        if args.headless {
            eprintln!("WARNING: Headless mode not yet implemented. Running with desktop UI.");
        }

        println!("Starting in server mode on port {}...", port);
        println!("API will be available at: http://localhost:{}/api/v1", port);
        println!("WebSocket at: ws://localhost:{}/api/v1/ws", port);
        println!("\nWARNING: No authentication enabled. Use on trusted networks only!");

        (rzem_ai_inference::shared::protocol::RuntimeConfig::server(port), Some(port))
    } else {
        // Local mode (default)
        (rzem_ai_inference::shared::protocol::RuntimeConfig::local(), None)
    };

    rzem_ai_inference::run_with_config(runtime_config, port)
}
