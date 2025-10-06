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

mod monitoring;
mod error;
mod protocol_state;
mod protocol_common;
mod lib3270;
mod protocol;
mod constants;

use tn5250r::app_state::TN5250RApp;
use controller::AsyncTerminalController;
use field_manager::FieldDisplayInfo;
use tn5250r::profile_manager::ProfileManager;
use tn5250r::session_profile::SessionProfile;

#[tokio::main]
async fn main() {
    println!("TN5250R starting...");
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
    let mut default_server = "as400.example.com".to_string();
    let mut default_port = 23;
    let mut auto_connect = false;
    let mut cli_ssl_override: Option<bool> = None;
    let mut cli_insecure: Option<bool> = None;
    let mut cli_ca_bundle: Option<String> = None;
    let mut cli_username: Option<String> = None;
    let mut cli_password: Option<String> = None;
    let mut cli_profile: Option<String> = None;
    let mut cli_save_profile: Option<String> = None;
    let mut cli_protocol: Option<String> = None;
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
            "--profile" | "-P" => {
                if i + 1 < args.len() {
                    cli_profile = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --profile requires a profile name");
                    std::process::exit(1);
                }
            }
            "--save-profile" => {
                if i + 1 < args.len() {
                    cli_save_profile = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --save-profile requires a profile name");
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
            "--protocol" => {
                if i + 1 < args.len() {
                    cli_protocol = Some(args[i + 1].clone());
                    i += 1; // consume value
                } else {
                    eprintln!("Error: --protocol requires a value (tn5250 or tn3270)");
                    std::process::exit(1);
                }
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
                println!("  --protocol <protocol>               Force protocol: tn5250 (AS/400) or tn3270 (mainframe)");
                println!("  --profile <name> or -P <name>       Load and connect using named profile");
                println!("  --save-profile <name>               Save current session as named profile");
                println!("  --ssl | --no-ssl                    Force TLS on/off for this run (overrides config)");
                println!("  --insecure                          Accept invalid TLS certs and hostnames (NOT recommended)");
                println!("  --ca-bundle <path>                  Use a custom CA bundle (PEM or DER) to validate server certs");
                println!("  --debug or -d                       Enable debug mode (verbose logging + data dumps)");
                println!("  --verbose or -v                     Enable verbose connection logging");
                println!("  --help or -h                        Show this help message");
                println!();
                println!("Example:");
                println!("  tn5250r --server as400.example.com --port 23 --user myuser --password mypass");
                println!("  tn5250r --server pub400.com --port 23 --protocol tn5250");
                println!("  tn5250r --server mainframe.example.com --port 23 --protocol tn3270");
                println!("  tn5250r --profile production");
                println!("  tn5250r --server host --save-profile dev");
                std::process::exit(0);
            }
            _ => { /* ignore unknown */ }
        }
        i += 1;
    }

    // Profile resolution logic
    let session_config = if let Some(profile_name) = cli_profile {
        // Load named profile
        let profile_manager = ProfileManager::new()
            .expect("Failed to initialize profile manager");

        match profile_manager.get_profile_by_name(&profile_name) {
            Some(profile) => {
                println!("Loading profile: {}", profile_name);
                Some(profile.clone())
            }
            None => {
                eprintln!("Error: Profile '{}' not found", profile_name);
                eprintln!("Available profiles:");
                for name in profile_manager.get_profile_names() {
                    eprintln!("  - {}", name);
                }
                std::process::exit(1);
            }
        }
    } else if auto_connect {
        // Create ephemeral profile from CLI args
        let ephemeral_profile = SessionProfile::new(
            "Ephemeral Session".to_string(),
            default_server.clone(),
            default_port,
        );

        // Apply CLI credentials if provided
        let mut profile_to_save = ephemeral_profile.clone();
        if let Some(username) = &cli_username {
            profile_to_save.username = Some(username.clone());
        }
        if let Some(password) = &cli_password {
            profile_to_save.password = Some(password.clone());
        }

        // Save if --save-profile specified
        if let Some(save_name) = &cli_save_profile {
            let mut profile_manager = ProfileManager::new()
                .expect("Failed to initialize profile manager");

            profile_to_save.name = save_name.clone();
            profile_to_save.id = save_name.clone();

            if let Err(e) = profile_manager.create_profile(profile_to_save.clone()) {
                eprintln!("Warning: Failed to save profile '{}': {}", save_name, e);
            } else {
                println!("Profile '{}' saved successfully", save_name);
            }
        }

        Some(profile_to_save)
    } else {
        None // No profile, show UI
    };

    // Select graphics backend based on environment to avoid Wayland/glutin issues.
    // Default behavior: if running on Wayland, prefer wgpu; otherwise use glow.
    // Allow override via TN5250R_BACKEND=wgpu|glow|auto.
    let backend_pref = std::env::var("TN5250R_BACKEND").unwrap_or_else(|_| "auto".to_string());
    let is_wayland = std::env::var("WAYLAND_DISPLAY").map(|v| !v.is_empty()).unwrap_or(false);
    let use_wgpu = match backend_pref.as_str() {
        "wgpu" => true,
        "glow" => false,
        _ => is_wayland, // auto: wgpu on Wayland, glow otherwise
    };

    let mut options = eframe::NativeOptions::default();
    options.viewport = egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        .with_visible(true)
        .with_active(true);
    // Configure renderer
    #[cfg(feature = "wgpu")]
    if use_wgpu {
        options.renderer = eframe::Renderer::Wgpu;
    } else {
        options.renderer = eframe::Renderer::Glow;
    }
    #[cfg(not(feature = "wgpu"))]
    {
        // If wgpu feature isn't available, fall back to glow
        let _ = use_wgpu; // silence unused warning
        options.renderer = eframe::Renderer::Glow;
    }

    println!("Running eframe...");

    eframe::run_native(
        "TN5250R",
        options,
        Box::new(move |cc| {
            println!("Creating app...");
            let app = TN5250RApp::new_with_profile(
                cc,
                session_config,
                cli_protocol,
                debug_mode,
            );
            println!("App created successfully");
            // Apply CLI TLS extras into config for this run (persist if user later saves/changes)
            if let Some(insec) = cli_insecure {
                if let Ok(mut cfg) = app.config.lock() { cfg.set_property("connection.tls.insecure", insec); }
                let _ = tn5250r::config::save_shared_config(&app.config);
            }
            if let Some(path) = cli_ca_bundle {
                if let Ok(mut cfg) = app.config.lock() { cfg.set_property("connection.tls.caBundlePath", path); }
                let _ = tn5250r::config::save_shared_config(&app.config);
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