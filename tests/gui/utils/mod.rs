//! Utility functions and helpers for GUI testing
//!
//! This module contains common utilities used across GUI test modules.
//! It is organized into focused submodules for better maintainability.

/// Test data generation utilities
pub mod test_data;

/// Assertion helpers for GUI testing
pub mod assertions;

/// Common test setup and teardown functions
pub mod setup;

/// Screenshot and visual comparison utilities
pub mod visual;

/// Performance measurement utilities for GUI tests
pub mod performance;
