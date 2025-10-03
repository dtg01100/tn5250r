//! Main application entry point for TN5250R
//!
//! This module handles the GUI application lifecycle and user interface.

use eframe::egui;

mod lib5250;
mod ansi_processor;
mod network;
mod terminal;
mod telnet_negotiation;
mod keyboard;
mod controller;
mod field_manager;
mod config;
mod monitoring;
mod error;
mod protocol_state;
mod protocol_common;
mod lib3270;
mod protocol;
mod app_state;
mod connection;
mod terminal_display;
mod input;
mod app;
mod ui;
mod constants;

use app_state::TN5250RApp;
use controller::AsyncTerminalController;
use field_manager::FieldDisplayInfo;

#[tokio::main]
async fn main() {
    // Install panic handler to log panics before crashing
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("!!! PANIC !!!");
        eprintln!("Program panicked: {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("Panic occurred in file '{}' at line {}", location.file(), location.line());
        }
        eprintln!("Stack backtrace:");
        eprintln!("{:?}", std::backtrace::Backtrace::capture());
    }));

    env_logger::init();

    // Initialize comprehensive monitoring system
    monitoring::init_monitoring();
    println!("MONITORING: System monitoring initialized");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut default_server = "example.system.com".to_string();
    let mut default_port = 23;
    let mut auto_connect = false;
    let mut cli_ssl_override: Option<bool> = None;
    let mut cli_insecure: Option<bool> = None;
    let mut cli_ca_bundle: Option<String> = None;
    let mut cli_username: Option<String> = None;
    let mut cli_password: Option<String> = None;
    let mut debug_mode = false;
    let mut verbose_mode = false;

    // Parse --server and --port options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--server" | "-s" => {
                if i + 1 < args.len() {
                    default_server = args[i + 1].clone();
                    auto_connect = true; // Auto-connect when server is specified
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --server requires a value");
                    std::process::exit(1);
                }
            }
            "--port" | "-p" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<u16>() {
                        Ok(p) => default_port = p,
                        Err(_) => {
                            eprintln!("Error: --port requires a numeric value");
                            std::process::exit(1);
                        }
                    }
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --port requires a value");
                    std::process::exit(1);
                }
            }
            "--ssl" => { cli_ssl_override = Some(true); }
            "--no-ssl" => { cli_ssl_override = Some(false); }
            "--insecure" => { cli_insecure = Some(true); }
            "--ca-bundle" => {
                if i + 1 < args.len() {
                    cli_ca_bundle = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --ca-bundle requires a path");
                    std::process::exit(1);
                }
            }
            "--user" | "-u" => {
                if i + 1 < args.len() {
                    cli_username = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --user requires a username");
                    std::process::exit(1);
                }
            }
            "--password" | "--pass" => {
                if i + 1 < args.len() {
                    cli_password = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --password requires a password");
                    std::process::exit(1);
                }
            }
            "--debug" | "-d" => {
                debug_mode = true;
                println!("DEBUG MODE ENABLED: Verbose logging and data dumps active");
            }
            "--verbose" | "-v" => {
                verbose_mode = true;
                println!("VERBOSE MODE: Detailed connection logs active");
            }
            "--help" | "-h" => {
                println!("TN5250R - IBM AS/400 Terminal Emulator");
                println!();
                println!("Usage: tn5250r [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --server <server> or -s <server>    Set the server to connect to and auto-connect");
                println!("  --port <port> or -p <port>          Set the port to connect to (default: 23)");
                println!("  --user <username> or -u <username>  AS/400 username for authentication (RFC 4777)");
                println!("  --password <password> or --pass     AS/400 password for authentication (RFC 4777)");
                println!("  --ssl | --no-ssl                    Force TLS on/off for this run (overrides config)");
                println!("  --insecure                          Accept invalid TLS certs and hostnames (NOT recommended)");
                println!("  --ca-bundle <path>                  Use a custom CA bundle (PEM or DER) to validate server certs");
                println!("  --debug or -d                       Enable debug mode (verbose logging + data dumps)");
                println!("  --verbose or -v                     Enable verbose connection logging");
                println!("  --help or -h                        Show this help message");
                println!();
                println!("Example:");
                println!("  tn5250r --server 10.100.200.1 --port 23 --user dave3 --password dave3");
                std::process::exit(0);
            }
            _ => { /* ignore unknown */ }
        }
        i += 1;
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "TN5250R",
        options,
        Box::new(move |cc| {
            let app = TN5250RApp::new_with_server(
                cc,
                default_server,
                default_port,
                auto_connect,
                cli_ssl_override,
                cli_username,
                cli_password,
                debug_mode,
            );
            // Apply CLI TLS extras into config for this run (persist if user later saves/changes)
            if let Some(insec) = cli_insecure {
                if let Ok(mut cfg) = app.config.lock() { cfg.set_property("connection.tls.insecure", insec); }
                let _ = config::save_shared_config(&app.config);
            }
            if let Some(path) = cli_ca_bundle {
                if let Ok(mut cfg) = app.config.lock() { cfg.set_property("connection.tls.caBundlePath", path); }
                let _ = config::save_shared_config(&app.config);
            }
            Ok(Box::new(app))
        }),
    )
    .expect("Failed to run TN5250R application");

    // Graceful shutdown with monitoring cleanup
    monitoring::shutdown_monitoring();
    println!("MONITORING: Application shutdown complete");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_initialization() {
        assert!(true);
    }
}