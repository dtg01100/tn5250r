//! Enhanced performance monitoring system for TN5250R
//!
//! This module provides comprehensive performance tracking, bottleneck detection,
//! and performance regression analysis for production operation.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use std::collections::{HashMap, VecDeque};
use super::{HealthStatus, ComponentHealthCheck};

/// Enhanced performance monitoring system
#[derive(Debug)]
pub struct PerformanceMonitor {
    /// Performance metrics
    pub metrics: PerformanceMetrics,
    /// Performance thresholds and configuration
    config: PerformanceConfig,
    /// Historical performance data for trend analysis
    history: std::sync::Mutex<VecDeque<PerformanceSnapshot>>,
    /// Performance tracking start time
    start_time: Instant,
}

/// Performance monitoring metrics
#[derive(Debug)]
pub struct PerformanceMetrics {
    /// Network performance metrics
    pub network: NetworkPerformanceMetrics,
    /// Terminal performance metrics
    pub terminal: TerminalPerformanceMetrics,
    /// Protocol processing metrics
    pub protocol: ProtocolPerformanceMetrics,
    /// Memory performance metrics
    pub memory: MemoryPerformanceMetrics,
    /// System performance metrics
    pub system: SystemPerformanceMetrics,
}

impl Clone for PerformanceMetrics {
    fn clone(&self) -> Self {
        Self {
            network: NetworkPerformanceMetrics {
                bytes_received_per_sec: AtomicU64::new(self.network.bytes_received_per_sec.load(Ordering::Relaxed)),
                bytes_sent_per_sec: AtomicU64::new(self.network.bytes_sent_per_sec.load(Ordering::Relaxed)),
                packets_received_per_sec: AtomicU64::new(self.network.packets_received_per_sec.load(Ordering::Relaxed)),
                packets_sent_per_sec: AtomicU64::new(self.network.packets_sent_per_sec.load(Ordering::Relaxed)),
                avg_connection_latency_us: AtomicU64::new(self.network.avg_connection_latency_us.load(Ordering::Relaxed)),
                connection_success_rate: AtomicU64::new(self.network.connection_success_rate.load(Ordering::Relaxed)),
                buffer_pool_efficiency: AtomicU64::new(self.network.buffer_pool_efficiency.load(Ordering::Relaxed)),
            },
            terminal: TerminalPerformanceMetrics {
                screen_updates_per_sec: AtomicU64::new(self.terminal.screen_updates_per_sec.load(Ordering::Relaxed)),
                character_writes_per_sec: AtomicU64::new(self.terminal.character_writes_per_sec.load(Ordering::Relaxed)),
                avg_screen_refresh_us: AtomicU64::new(self.terminal.avg_screen_refresh_us.load(Ordering::Relaxed)),
                field_operations_per_sec: AtomicU64::new(self.terminal.field_operations_per_sec.load(Ordering::Relaxed)),
                cache_hit_rate: AtomicU64::new(self.terminal.cache_hit_rate.load(Ordering::Relaxed)),
                avg_ui_render_time_us: AtomicU64::new(self.terminal.avg_ui_render_time_us.load(Ordering::Relaxed)),
            },
            protocol: ProtocolPerformanceMetrics {
                commands_processed_per_sec: AtomicU64::new(self.protocol.commands_processed_per_sec.load(Ordering::Relaxed)),
                ebcdic_conversions_per_sec: AtomicU64::new(self.protocol.ebcdic_conversions_per_sec.load(Ordering::Relaxed)),
                avg_command_processing_us: AtomicU64::new(self.protocol.avg_command_processing_us.load(Ordering::Relaxed)),
                negotiation_success_rate: AtomicU64::new(self.protocol.negotiation_success_rate.load(Ordering::Relaxed)),
                field_detections_per_sec: AtomicU64::new(self.protocol.field_detections_per_sec.load(Ordering::Relaxed)),
                structured_fields_per_sec: AtomicU64::new(self.protocol.structured_fields_per_sec.load(Ordering::Relaxed)),
            },
            memory: MemoryPerformanceMetrics {
                allocations_per_sec: AtomicU64::new(self.memory.allocations_per_sec.load(Ordering::Relaxed)),
                deallocations_per_sec: AtomicU64::new(self.memory.deallocations_per_sec.load(Ordering::Relaxed)),
                current_memory_usage: AtomicU64::new(self.memory.current_memory_usage.load(Ordering::Relaxed)),
                peak_memory_usage: AtomicU64::new(self.memory.peak_memory_usage.load(Ordering::Relaxed)),
                fragmentation_ratio: AtomicU64::new(self.memory.fragmentation_ratio.load(Ordering::Relaxed)),
                buffer_pool_utilization: AtomicU64::new(self.memory.buffer_pool_utilization.load(Ordering::Relaxed)),
            },
            system: SystemPerformanceMetrics {
                total_cpu_time_us: AtomicU64::new(self.system.total_cpu_time_us.load(Ordering::Relaxed)),
                avg_cpu_usage_percent: AtomicU64::new(self.system.avg_cpu_usage_percent.load(Ordering::Relaxed)),
                thread_count: AtomicU64::new(self.system.thread_count.load(Ordering::Relaxed)),
                context_switches_per_sec: AtomicU64::new(self.system.context_switches_per_sec.load(Ordering::Relaxed)),
                system_load_1min: AtomicU64::new(self.system.system_load_1min.load(Ordering::Relaxed)),
                system_load_5min: AtomicU64::new(self.system.system_load_5min.load(Ordering::Relaxed)),
            },
        }
    }
}

/// Network performance metrics
#[derive(Debug)]
pub struct NetworkPerformanceMetrics {
    /// Bytes received per second
    pub bytes_received_per_sec: AtomicU64,
    /// Bytes sent per second
    pub bytes_sent_per_sec: AtomicU64,
    /// Packets received per second
    pub packets_received_per_sec: AtomicU64,
    /// Packets sent per second
    pub packets_sent_per_sec: AtomicU64,
    /// Average connection latency in microseconds
    pub avg_connection_latency_us: AtomicU64,
    /// Connection establishment success rate (percentage)
    pub connection_success_rate: AtomicU64,
    /// Network buffer pool efficiency (percentage)
    pub buffer_pool_efficiency: AtomicU64,
}

/// Terminal performance metrics
#[derive(Debug)]
pub struct TerminalPerformanceMetrics {
    /// Screen updates per second
    pub screen_updates_per_sec: AtomicU64,
    /// Character writes per second
    pub character_writes_per_sec: AtomicU64,
    /// Average screen refresh time in microseconds
    pub avg_screen_refresh_us: AtomicU64,
    /// Field processing operations per second
    pub field_operations_per_sec: AtomicU64,
    /// Cache hit rate (percentage)
    pub cache_hit_rate: AtomicU64,
    /// UI rendering time in microseconds
    pub avg_ui_render_time_us: AtomicU64,
}

/// Protocol processing metrics
#[derive(Debug)]
pub struct ProtocolPerformanceMetrics {
    /// Commands processed per second
    pub commands_processed_per_sec: AtomicU64,
    /// EBCDIC conversions per second
    pub ebcdic_conversions_per_sec: AtomicU64,
    /// Average command processing time in microseconds
    pub avg_command_processing_us: AtomicU64,
    /// Protocol negotiation success rate (percentage)
    pub negotiation_success_rate: AtomicU64,
    /// Field detection operations per second
    pub field_detections_per_sec: AtomicU64,
    /// Structured field processing per second
    pub structured_fields_per_sec: AtomicU64,
}

/// Memory performance metrics
#[derive(Debug)]
pub struct MemoryPerformanceMetrics {
    /// Memory allocations per second
    pub allocations_per_sec: AtomicU64,
    /// Memory deallocations per second
    pub deallocations_per_sec: AtomicU64,
    /// Current memory usage in bytes
    pub current_memory_usage: AtomicU64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: AtomicU64,
    /// Memory fragmentation ratio (0.0-1.0)
    pub fragmentation_ratio: AtomicU64,
    /// Buffer pool utilization (percentage)
    pub buffer_pool_utilization: AtomicU64,
}

/// System performance metrics
#[derive(Debug)]
pub struct SystemPerformanceMetrics {
    /// Total CPU time used in microseconds
    pub total_cpu_time_us: AtomicU64,
    /// Average CPU usage percentage
    pub avg_cpu_usage_percent: AtomicU64,
    /// Thread count
    pub thread_count: AtomicU64,
    /// Context switches per second
    pub context_switches_per_sec: AtomicU64,
    /// System load average (1-minute)
    pub system_load_1min: AtomicU64,
    /// System load average (5-minute)
    pub system_load_5min: AtomicU64,
}

/// Performance configuration and thresholds
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Enable detailed performance tracking
    pub enable_detailed_tracking: bool,
    /// Performance history retention count
    pub history_retention_count: usize,
    /// Performance snapshot interval in seconds
    pub snapshot_interval_seconds: u64,
    /// Warning threshold for high CPU usage (percentage)
    pub cpu_warning_threshold: f64,
    /// Critical threshold for high CPU usage (percentage)
    pub cpu_critical_threshold: f64,
    /// Warning threshold for memory usage (percentage)
    pub memory_warning_threshold: f64,
    /// Critical threshold for memory usage (percentage)
    pub memory_critical_threshold: f64,
    /// Warning threshold for network latency (microseconds)
    pub latency_warning_threshold_us: u64,
    /// Critical threshold for network latency (microseconds)
    pub latency_critical_threshold_us: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_detailed_tracking: true,
            // Keep fewer snapshots to lower memory in constrained environments
            history_retention_count: 30,
            snapshot_interval_seconds: 60,
            cpu_warning_threshold: 70.0,
            cpu_critical_threshold: 90.0,
            memory_warning_threshold: 80.0,
            memory_critical_threshold: 95.0,
            latency_warning_threshold_us: 100_000, // 100ms
            latency_critical_threshold_us: 500_000, // 500ms
        }
    }
}

/// Performance snapshot for historical analysis
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Snapshot timestamp
    pub timestamp: Instant,
    /// Performance metrics at this point in time
    pub metrics: PerformanceMetrics,
    /// System load at snapshot time
    pub system_load: f64,
    /// Memory usage at snapshot time
    pub memory_usage: u64,
}

impl PerformanceMonitor {
    /// Create a new performance monitor instance
    pub fn new() -> Self {
        Self {
            metrics: PerformanceMetrics {
                network: NetworkPerformanceMetrics {
                    bytes_received_per_sec: AtomicU64::new(0),
                    bytes_sent_per_sec: AtomicU64::new(0),
                    packets_received_per_sec: AtomicU64::new(0),
                    packets_sent_per_sec: AtomicU64::new(0),
                    avg_connection_latency_us: AtomicU64::new(0),
                    connection_success_rate: AtomicU64::new(100), // Start at 100%
                    buffer_pool_efficiency: AtomicU64::new(100),   // Start at 100%
                },
                terminal: TerminalPerformanceMetrics {
                    screen_updates_per_sec: AtomicU64::new(0),
                    character_writes_per_sec: AtomicU64::new(0),
                    avg_screen_refresh_us: AtomicU64::new(0),
                    field_operations_per_sec: AtomicU64::new(0),
                    cache_hit_rate: AtomicU64::new(100), // Start at 100%
                    avg_ui_render_time_us: AtomicU64::new(0),
                },
                protocol: ProtocolPerformanceMetrics {
                    commands_processed_per_sec: AtomicU64::new(0),
                    ebcdic_conversions_per_sec: AtomicU64::new(0),
                    avg_command_processing_us: AtomicU64::new(0),
                    negotiation_success_rate: AtomicU64::new(100), // Start at 100%
                    field_detections_per_sec: AtomicU64::new(0),
                    structured_fields_per_sec: AtomicU64::new(0),
                },
                memory: MemoryPerformanceMetrics {
                    allocations_per_sec: AtomicU64::new(0),
                    deallocations_per_sec: AtomicU64::new(0),
                    current_memory_usage: AtomicU64::new(0),
                    peak_memory_usage: AtomicU64::new(0),
                    fragmentation_ratio: AtomicU64::new(0),
                    buffer_pool_utilization: AtomicU64::new(0),
                },
                system: SystemPerformanceMetrics {
                    total_cpu_time_us: AtomicU64::new(0),
                    avg_cpu_usage_percent: AtomicU64::new(0),
                    thread_count: AtomicU64::new(1), // Main thread
                    context_switches_per_sec: AtomicU64::new(0),
                    system_load_1min: AtomicU64::new(0),
                    system_load_5min: AtomicU64::new(0),
                },
            },
            config: PerformanceConfig::default(),
            history: std::sync::Mutex::new(VecDeque::new()),
            start_time: Instant::now(),
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self { Self::new() }
}

impl PerformanceMonitor {
    /// Check performance health and detect issues
    pub fn check_performance_health(&self) -> Result<ComponentHealthCheck, String> {
        let mut details = HashMap::new();
        let mut issues = Vec::new();
        let mut overall_status = HealthStatus::Healthy;

        // Check CPU usage
        let cpu_usage = self.metrics.system.avg_cpu_usage_percent.load(Ordering::Relaxed) as f64;
        details.insert("cpu_usage_percent".to_string(), format!("{cpu_usage:.1}"));

        if cpu_usage > self.config.cpu_critical_threshold {
            issues.push(format!("CPU usage critical: {cpu_usage:.1}%"));
            overall_status = HealthStatus::Critical;
        } else if cpu_usage > self.config.cpu_warning_threshold {
            issues.push(format!("CPU usage high: {cpu_usage:.1}%"));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check memory usage
        let memory_usage = self.get_memory_usage_percent();
        details.insert("memory_usage_percent".to_string(), format!("{memory_usage:.1}"));

        if memory_usage > self.config.memory_critical_threshold {
            issues.push(format!("Memory usage critical: {memory_usage:.1}%"));
            overall_status = HealthStatus::Critical;
        } else if memory_usage > self.config.memory_warning_threshold {
            issues.push(format!("Memory usage high: {memory_usage:.1}%"));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check network latency
        let latency = self.metrics.network.avg_connection_latency_us.load(Ordering::Relaxed);
        details.insert("network_latency_us".to_string(), latency.to_string());

        if latency > self.config.latency_critical_threshold_us {
            issues.push(format!("Network latency critical: {latency} μs"));
            overall_status = HealthStatus::Critical;
        } else if latency > self.config.latency_warning_threshold_us {
            issues.push(format!("Network latency high: {latency} μs"));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check connection success rate
        let conn_success_rate = self.metrics.network.connection_success_rate.load(Ordering::Relaxed);
        details.insert("connection_success_rate".to_string(), format!("{conn_success_rate}%"));

        if conn_success_rate < 90 {
            issues.push(format!("Connection success rate low: {conn_success_rate}%"));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        // Check for performance regressions
        if let Some(regression) = self.detect_performance_regression() {
            issues.push(format!("Performance regression detected: {regression}"));
            if overall_status == HealthStatus::Healthy {
                overall_status = HealthStatus::Warning;
            }
        }

        let message = if issues.is_empty() {
            "All performance metrics within acceptable ranges".to_string()
        } else {
            format!("Performance issues detected: {}", issues.join(", "))
        };

        Ok(ComponentHealthCheck {
            status: overall_status,
            message,
            details,
        })
    }

    /// Update performance metrics from the global performance metrics
    pub fn update_metrics(&self) {
        // This method would integrate with the existing performance_metrics.rs
        // For now, we'll update some basic metrics

        let uptime = self.start_time.elapsed();

        // Update system metrics (simplified)
        self.metrics.system.thread_count.store(4, Ordering::Relaxed); // Approximate thread count
        self.metrics.system.total_cpu_time_us.store(
            uptime.as_micros() as u64,
            Ordering::Relaxed
        );

        // Update memory metrics (simplified)
        self.metrics.memory.current_memory_usage.store(50 * 1024 * 1024, Ordering::Relaxed); // 50MB
        self.metrics.memory.peak_memory_usage.store(100 * 1024 * 1024, Ordering::Relaxed);   // 100MB

        // Take performance snapshot if needed
        if uptime.as_secs() % self.config.snapshot_interval_seconds == 0 {
            self.take_performance_snapshot();
        }
    }

    /// Take a performance snapshot for historical analysis
    fn take_performance_snapshot(&self) {
        let snapshot = PerformanceSnapshot {
            timestamp: Instant::now(),
            metrics: self.metrics.clone(),
            system_load: self.get_system_load(),
            memory_usage: self.metrics.memory.current_memory_usage.load(Ordering::Relaxed),
        };

        if let Ok(mut history) = self.history.lock() {
            history.push_back(snapshot);

            // Trim history to retain only the configured count
            while history.len() > self.config.history_retention_count {
                history.pop_front();
            }
        }
    }

    /// Detect performance regressions by analyzing historical data
    fn detect_performance_regression(&self) -> Option<String> {
        if let Ok(history) = self.history.lock() {
            if history.len() < 2 {
                return None; // Need at least 2 snapshots for comparison
            }

            let recent = history.back().unwrap();
            let baseline = history.front().unwrap();

            // Compare key metrics
            let latency_recent = recent.metrics.network.avg_connection_latency_us.load(Ordering::Relaxed);
            let latency_baseline = baseline.metrics.network.avg_connection_latency_us.load(Ordering::Relaxed);

            if latency_recent > latency_baseline * 2 {
                return Some(format!("Network latency increased from {latency_baseline} μs to {latency_recent} μs"));
            }

            let cpu_recent = recent.metrics.system.avg_cpu_usage_percent.load(Ordering::Relaxed) as f64;
            let cpu_baseline = baseline.metrics.system.avg_cpu_usage_percent.load(Ordering::Relaxed) as f64;

            if cpu_recent > cpu_baseline * 1.5 {
                return Some(format!("CPU usage increased from {cpu_baseline:.1}% to {cpu_recent:.1}%"));
            }
        }

        None
    }

    /// Get current memory usage as percentage
    fn get_memory_usage_percent(&self) -> f64 {
        let current = self.metrics.memory.current_memory_usage.load(Ordering::Relaxed);
        let peak = self.metrics.memory.peak_memory_usage.load(Ordering::Relaxed);

        if peak == 0 {
            0.0
        } else {
            (current as f64 / peak as f64) * 100.0
        }
    }

    /// Get current system load (simplified implementation)
    fn get_system_load(&self) -> f64 {
        // In a real implementation, you would get actual system load
        // For now, return a mock value based on CPU usage
        self.metrics.system.avg_cpu_usage_percent.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &PerformanceMetrics {
        &self.metrics
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("=== Performance Monitoring Report ===\n\n");

        // Network performance
        report.push_str("Network Performance:\n");
        report.push_str(&format!("  Bytes/sec (RX/TX): {}/{}\n",
            self.metrics.network.bytes_received_per_sec.load(Ordering::Relaxed),
            self.metrics.network.bytes_sent_per_sec.load(Ordering::Relaxed)));
        report.push_str(&format!("  Packets/sec (RX/TX): {}/{}\n",
            self.metrics.network.packets_received_per_sec.load(Ordering::Relaxed),
            self.metrics.network.packets_sent_per_sec.load(Ordering::Relaxed)));
        report.push_str(&format!("  Avg Latency: {} μs\n",
            self.metrics.network.avg_connection_latency_us.load(Ordering::Relaxed)));
        report.push_str(&format!("  Connection Success Rate: {}%\n",
            self.metrics.network.connection_success_rate.load(Ordering::Relaxed)));

        // Terminal performance
        report.push_str("\nTerminal Performance:\n");
        report.push_str(&format!("  Screen Updates/sec: {}\n",
            self.metrics.terminal.screen_updates_per_sec.load(Ordering::Relaxed)));
        report.push_str(&format!("  Character Writes/sec: {}\n",
            self.metrics.terminal.character_writes_per_sec.load(Ordering::Relaxed)));
        report.push_str(&format!("  Cache Hit Rate: {}%\n",
            self.metrics.terminal.cache_hit_rate.load(Ordering::Relaxed)));

        // Protocol performance
        report.push_str("\nProtocol Performance:\n");
        report.push_str(&format!("  Commands/sec: {}\n",
            self.metrics.protocol.commands_processed_per_sec.load(Ordering::Relaxed)));
        report.push_str(&format!("  EBCDIC Conversions/sec: {}\n",
            self.metrics.protocol.ebcdic_conversions_per_sec.load(Ordering::Relaxed)));

        // Memory performance
        report.push_str("\nMemory Performance:\n");
        report.push_str(&format!("  Current Usage: {} MB\n",
            self.metrics.memory.current_memory_usage.load(Ordering::Relaxed) / (1024 * 1024)));
        report.push_str(&format!("  Peak Usage: {} MB\n",
            self.metrics.memory.peak_memory_usage.load(Ordering::Relaxed) / (1024 * 1024)));
        report.push_str(&format!("  Buffer Pool Utilization: {}%\n",
            self.metrics.memory.buffer_pool_utilization.load(Ordering::Relaxed)));

        // System performance
        report.push_str("\nSystem Performance:\n");
        report.push_str(&format!("  Thread Count: {}\n",
            self.metrics.system.thread_count.load(Ordering::Relaxed)));
        report.push_str(&format!("  CPU Usage: {}%\n",
            self.metrics.system.avg_cpu_usage_percent.load(Ordering::Relaxed)));

        report
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            network: NetworkPerformanceMetrics {
                bytes_received_per_sec: AtomicU64::new(0),
                bytes_sent_per_sec: AtomicU64::new(0),
                packets_received_per_sec: AtomicU64::new(0),
                packets_sent_per_sec: AtomicU64::new(0),
                avg_connection_latency_us: AtomicU64::new(0),
                connection_success_rate: AtomicU64::new(100),
                buffer_pool_efficiency: AtomicU64::new(100),
            },
            terminal: TerminalPerformanceMetrics {
                screen_updates_per_sec: AtomicU64::new(0),
                character_writes_per_sec: AtomicU64::new(0),
                avg_screen_refresh_us: AtomicU64::new(0),
                field_operations_per_sec: AtomicU64::new(0),
                cache_hit_rate: AtomicU64::new(100),
                avg_ui_render_time_us: AtomicU64::new(0),
            },
            protocol: ProtocolPerformanceMetrics {
                commands_processed_per_sec: AtomicU64::new(0),
                ebcdic_conversions_per_sec: AtomicU64::new(0),
                avg_command_processing_us: AtomicU64::new(0),
                negotiation_success_rate: AtomicU64::new(100),
                field_detections_per_sec: AtomicU64::new(0),
                structured_fields_per_sec: AtomicU64::new(0),
            },
            memory: MemoryPerformanceMetrics {
                allocations_per_sec: AtomicU64::new(0),
                deallocations_per_sec: AtomicU64::new(0),
                current_memory_usage: AtomicU64::new(0),
                peak_memory_usage: AtomicU64::new(0),
                fragmentation_ratio: AtomicU64::new(0),
                buffer_pool_utilization: AtomicU64::new(0),
            },
            system: SystemPerformanceMetrics {
                total_cpu_time_us: AtomicU64::new(0),
                avg_cpu_usage_percent: AtomicU64::new(0),
                thread_count: AtomicU64::new(1),
                context_switches_per_sec: AtomicU64::new(0),
                system_load_1min: AtomicU64::new(0),
                system_load_5min: AtomicU64::new(0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new();
        assert_eq!(monitor.metrics.network.connection_success_rate.load(Ordering::Relaxed), 100);
        assert_eq!(monitor.metrics.terminal.cache_hit_rate.load(Ordering::Relaxed), 100);
    }

    #[test]
    fn test_performance_config_default() {
        let config = PerformanceConfig::default();
        assert_eq!(config.cpu_warning_threshold, 70.0);
        assert_eq!(config.memory_critical_threshold, 95.0);
        assert_eq!(config.latency_warning_threshold_us, 100_000);
    }

    #[test]
    fn test_memory_usage_calculation() {
        let monitor = PerformanceMonitor::new();
        monitor.metrics.memory.current_memory_usage.store(50 * 1024 * 1024, Ordering::Relaxed);
        monitor.metrics.memory.peak_memory_usage.store(100 * 1024 * 1024, Ordering::Relaxed);

        let usage_percent = monitor.get_memory_usage_percent();
        assert_eq!(usage_percent, 50.0);
    }
}