pub mod components;
pub mod integration;
// mod mocks; // TODO: Create mocks module
// mod utils; // TODO: Create utils module
pub mod visual;

use std::env;

fn main() {
    // Set up test environment
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    // Run all GUI tests
    println!("Running TN5250R GUI Test Suite...");

    // Note: Individual test modules contain #[cfg(test)] blocks
    // This main function serves as an entry point for CI/CD
    println!("GUI test suite initialized. Run with: cargo test --test gui");
}