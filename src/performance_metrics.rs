//! Performance monitoring and metrics for the tn5250r application
//!
//! This module provides comprehensive performance tracking capabilities
//! to monitor and analyze the effectiveness of performance optimizations.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

/// Global performance metrics instance
static GLOBAL_METRICS: once_cell::sync::Lazy<Arc<PerformanceMetrics>> =
    once_cell::sync::Lazy::new(|| Arc::new(PerformanceMetrics::new()));

/// Performance metrics structure
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Network I/O metrics
    pub network_metrics: NetworkMetrics,
    /// Terminal/screen buffer metrics
    pub terminal_metrics: TerminalMetrics,
    /// Protocol processing metrics
    pub protocol_metrics: ProtocolMetrics,
    /// Memory allocation metrics
    pub memory_metrics: MemoryMetrics,
    /// General timing metrics
    pub timing_metrics: TimingMetrics,
}

/// Network I/O performance metrics
#[derive(Debug)]
pub struct NetworkMetrics {
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
    pub packets_received: AtomicU64,
    pub packets_sent: AtomicU64,
    pub connection_count: AtomicU64,
    pub buffer_pool_hits: AtomicU64,
    pub buffer_pool_misses: AtomicU64,
}

/// Terminal/screen buffer performance metrics
#[derive(Debug)]
pub struct TerminalMetrics {
    pub screen_updates: AtomicU64,
    pub character_writes: AtomicU64,
    pub buffer_clears: AtomicU64,
    pub region_operations: AtomicU64,
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
}

/// Protocol processing performance metrics
#[derive(Debug)]
pub struct ProtocolMetrics {
    pub commands_processed: AtomicU64,
    pub ebcdic_conversions: AtomicU64,
    pub field_detections: AtomicU64,
    pub structured_fields: AtomicU64,
    pub errors_encountered: AtomicU64,
}

/// Memory allocation performance metrics
#[derive(Debug)]
pub struct MemoryMetrics {
    pub allocations: AtomicU64,
    pub deallocations: AtomicU64,
    pub buffer_reuses: AtomicU64,
    pub peak_memory_usage: AtomicU64,
    pub current_memory_usage: AtomicU64,
}

/// General timing performance metrics
#[derive(Debug)]
pub struct TimingMetrics {
    pub total_uptime: AtomicU64,
    pub average_command_time: AtomicU64,
    pub max_command_time: AtomicU64,
    pub total_commands: AtomicU64,
}

impl PerformanceMetrics {
    /// Create a new performance metrics instance
    pub fn new() -> Self {
        Self {
            network_metrics: NetworkMetrics {
                bytes_received: AtomicU64::new(0),
                bytes_sent: AtomicU64::new(0),
                packets_received: AtomicU64::new(0),
                packets_sent: AtomicU64::new(0),
                connection_count: AtomicU64::new(0),
                buffer_pool_hits: AtomicU64::new(0),
                buffer_pool_misses: AtomicU64::new(0),
            },
            terminal_metrics: TerminalMetrics {
                screen_updates: AtomicU64::new(0),
                character_writes: AtomicU64::new(0),
                buffer_clears: AtomicU64::new(0),
                region_operations: AtomicU64::new(0),
                cache_hits: AtomicU64::new(0),
                cache_misses: AtomicU64::new(0),
            },
            protocol_metrics: ProtocolMetrics {
                commands_processed: AtomicU64::new(0),
                ebcdic_conversions: AtomicU64::new(0),
                field_detections: AtomicU64::new(0),
                structured_fields: AtomicU64::new(0),
                errors_encountered: AtomicU64::new(0),
            },
            memory_metrics: MemoryMetrics {
                allocations: AtomicU64::new(0),
                deallocations: AtomicU64::new(0),
                buffer_reuses: AtomicU64::new(0),
                peak_memory_usage: AtomicU64::new(0),
                current_memory_usage: AtomicU64::new(0),
            },
            timing_metrics: TimingMetrics {
                total_uptime: AtomicU64::new(0),
                average_command_time: AtomicU64::new(0),
                max_command_time: AtomicU64::new(0),
                total_commands: AtomicU64::new(0),
            },
        }
    }

    /// Get the global performance metrics instance
    pub fn global() -> &'static Arc<PerformanceMetrics> {
        &GLOBAL_METRICS
    }

    /// Reset all metrics to zero
    pub fn reset(&self) {
        self.network_metrics.bytes_received.store(0, Ordering::Relaxed);
        self.network_metrics.bytes_sent.store(0, Ordering::Relaxed);
        self.network_metrics.packets_received.store(0, Ordering::Relaxed);
        self.network_metrics.packets_sent.store(0, Ordering::Relaxed);
        self.network_metrics.connection_count.store(0, Ordering::Relaxed);
        self.network_metrics.buffer_pool_hits.store(0, Ordering::Relaxed);
        self.network_metrics.buffer_pool_misses.store(0, Ordering::Relaxed);

        self.terminal_metrics.screen_updates.store(0, Ordering::Relaxed);
        self.terminal_metrics.character_writes.store(0, Ordering::Relaxed);
        self.terminal_metrics.buffer_clears.store(0, Ordering::Relaxed);
        self.terminal_metrics.region_operations.store(0, Ordering::Relaxed);
        self.terminal_metrics.cache_hits.store(0, Ordering::Relaxed);
        self.terminal_metrics.cache_misses.store(0, Ordering::Relaxed);

        self.protocol_metrics.commands_processed.store(0, Ordering::Relaxed);
        self.protocol_metrics.ebcdic_conversions.store(0, Ordering::Relaxed);
        self.protocol_metrics.field_detections.store(0, Ordering::Relaxed);
        self.protocol_metrics.structured_fields.store(0, Ordering::Relaxed);
        self.protocol_metrics.errors_encountered.store(0, Ordering::Relaxed);

        self.memory_metrics.allocations.store(0, Ordering::Relaxed);
        self.memory_metrics.deallocations.store(0, Ordering::Relaxed);
        self.memory_metrics.buffer_reuses.store(0, Ordering::Relaxed);
        self.memory_metrics.peak_memory_usage.store(0, Ordering::Relaxed);
        self.memory_metrics.current_memory_usage.store(0, Ordering::Relaxed);

        self.timing_metrics.total_uptime.store(0, Ordering::Relaxed);
        self.timing_metrics.average_command_time.store(0, Ordering::Relaxed);
        self.timing_metrics.max_command_time.store(0, Ordering::Relaxed);
        self.timing_metrics.total_commands.store(0, Ordering::Relaxed);
    }

    /// Generate a performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Performance Metrics Report ===\n\n");

        // Network metrics
        report.push_str("Network I/O:\n");
        report.push_str(&format!("  Bytes received: {}\n", self.network_metrics.bytes_received.load(Ordering::Relaxed)));
        report.push_str(&format!("  Bytes sent: {}\n", self.network_metrics.bytes_sent.load(Ordering::Relaxed)));
        report.push_str(&format!("  Packets received: {}\n", self.network_metrics.packets_received.load(Ordering::Relaxed)));
        report.push_str(&format!("  Packets sent: {}\n", self.network_metrics.packets_sent.load(Ordering::Relaxed)));
        report.push_str(&format!("  Connections: {}\n", self.network_metrics.connection_count.load(Ordering::Relaxed)));
        let buffer_pool_efficiency = if self.network_metrics.buffer_pool_hits.load(Ordering::Relaxed) + self.network_metrics.buffer_pool_misses.load(Ordering::Relaxed) > 0 {
            (self.network_metrics.buffer_pool_hits.load(Ordering::Relaxed) as f64 /
             (self.network_metrics.buffer_pool_hits.load(Ordering::Relaxed) + self.network_metrics.buffer_pool_misses.load(Ordering::Relaxed)) as f64) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("  Buffer pool efficiency: {buffer_pool_efficiency:.1}%\n"));

        // Terminal metrics
        report.push_str("\nTerminal Operations:\n");
        report.push_str(&format!("  Screen updates: {}\n", self.terminal_metrics.screen_updates.load(Ordering::Relaxed)));
        report.push_str(&format!("  Character writes: {}\n", self.terminal_metrics.character_writes.load(Ordering::Relaxed)));
        report.push_str(&format!("  Buffer clears: {}\n", self.terminal_metrics.buffer_clears.load(Ordering::Relaxed)));
        report.push_str(&format!("  Region operations: {}\n", self.terminal_metrics.region_operations.load(Ordering::Relaxed)));
        let cache_efficiency = if self.terminal_metrics.cache_hits.load(Ordering::Relaxed) + self.terminal_metrics.cache_misses.load(Ordering::Relaxed) > 0 {
            (self.terminal_metrics.cache_hits.load(Ordering::Relaxed) as f64 /
             (self.terminal_metrics.cache_hits.load(Ordering::Relaxed) + self.terminal_metrics.cache_misses.load(Ordering::Relaxed)) as f64) * 100.0
        } else {
            0.0
        };
        report.push_str(&format!("  Cache efficiency: {cache_efficiency:.1}%\n"));

        // Protocol metrics
        report.push_str("\nProtocol Processing:\n");
        report.push_str(&format!("  Commands processed: {}\n", self.protocol_metrics.commands_processed.load(Ordering::Relaxed)));
        report.push_str(&format!("  EBCDIC conversions: {}\n", self.protocol_metrics.ebcdic_conversions.load(Ordering::Relaxed)));
        report.push_str(&format!("  Field detections: {}\n", self.protocol_metrics.field_detections.load(Ordering::Relaxed)));
        report.push_str(&format!("  Structured fields: {}\n", self.protocol_metrics.structured_fields.load(Ordering::Relaxed)));
        report.push_str(&format!("  Errors encountered: {}\n", self.protocol_metrics.errors_encountered.load(Ordering::Relaxed)));

        // Memory metrics
        report.push_str("\nMemory Management:\n");
        report.push_str(&format!("  Allocations: {}\n", self.memory_metrics.allocations.load(Ordering::Relaxed)));
        report.push_str(&format!("  Deallocations: {}\n", self.memory_metrics.deallocations.load(Ordering::Relaxed)));
        report.push_str(&format!("  Buffer reuses: {}\n", self.memory_metrics.buffer_reuses.load(Ordering::Relaxed)));
        report.push_str(&format!("  Peak memory usage: {} bytes\n", self.memory_metrics.peak_memory_usage.load(Ordering::Relaxed)));
        report.push_str(&format!("  Current memory usage: {} bytes\n", self.memory_metrics.current_memory_usage.load(Ordering::Relaxed)));

        // Timing metrics
        report.push_str("\nTiming Metrics:\n");
        let uptime_secs = self.timing_metrics.total_uptime.load(Ordering::Relaxed);
        report.push_str(&format!("  Total uptime: {uptime_secs} seconds\n"));
        let total_commands = self.timing_metrics.total_commands.load(Ordering::Relaxed);
        if total_commands > 0 {
            let avg_time_ns = self.timing_metrics.average_command_time.load(Ordering::Relaxed);
            let max_time_ns = self.timing_metrics.max_command_time.load(Ordering::Relaxed);
            report.push_str(&format!("  Average command time: {avg_time_ns} ns\n"));
            report.push_str(&format!("  Max command time: {max_time_ns} ns\n"));
            report.push_str(&format!("  Commands per second: {:.1}\n", total_commands as f64 / uptime_secs as f64));
        }

        report
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics::new()
    }
}

/// Performance timing helper
pub struct PerformanceTimer {
    start_time: Instant,
    metric: &'static AtomicU64,
}

impl PerformanceTimer {
    /// Start timing a performance metric
    pub fn start(metric: &'static AtomicU64) -> Self {
        Self {
            start_time: Instant::now(),
            metric,
        }
    }
}

impl Drop for PerformanceTimer {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed().as_nanos() as u64;
        self.metric.fetch_add(elapsed, Ordering::Relaxed);
    }
}

// Performance monitoring macros - exported at crate level
// These macros provide convenient access to performance metrics

/// Increment a performance counter
#[macro_export]
macro_rules! perf_counter {
    ($category:ident, $field:ident) => {
        $crate::performance_metrics::PerformanceMetrics::global()
            .$category
            .$field
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    };
}

/// Start a performance timer
#[macro_export]
macro_rules! perf_timer {
    ($category:ident, $field:ident) => {
        $crate::performance_metrics::PerformanceTimer::start(
            &$crate::performance_metrics::PerformanceMetrics::global()
                .$category
                .$field
        )
    };
}

/// Add a value to a performance metric
#[macro_export]
macro_rules! perf_add {
    ($category:ident, $field:ident, $value:expr) => {
        $crate::performance_metrics::PerformanceMetrics::global()
            .$category
            .$field
            .fetch_add($value, std::sync::atomic::Ordering::Relaxed)
    };
}