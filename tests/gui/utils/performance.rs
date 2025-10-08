//! Performance measurement utilities for GUI tests
//!
//! This module provides utilities for measuring and analyzing GUI performance
//! metrics such as frame rates, rendering times, and memory usage.

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Performance measurement result
#[derive(Debug, Clone)]
pub struct PerformanceResult {
    pub operation: String,
    pub duration: Duration,
    pub start_time: Instant,
    pub end_time: Instant,
    pub metadata: HashMap<String, String>,
}

impl PerformanceResult {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            duration: Duration::default(),
            start_time: Instant::now(),
            end_time: Instant::now(),
            metadata: HashMap::new(),
        }
    }

    pub fn complete(mut self) -> Self {
        self.end_time = Instant::now();
        self.duration = self.end_time.duration_since(self.start_time);
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Performance timer for measuring operation duration
pub struct PerformanceTimer {
    start_time: Instant,
    operation: String,
    metadata: HashMap<String, String>,
}

impl PerformanceTimer {
    pub fn start(operation: &str) -> Self {
        println!("⏱️  Starting performance measurement for: {}", operation);
        Self {
            start_time: Instant::now(),
            operation: operation.to_string(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn stop(self) -> PerformanceResult {
        let mut result = PerformanceResult::new(&self.operation);
        result.end_time = Instant::now();
        result.duration = result.end_time.duration_since(self.start_time);
        result.metadata = self.metadata;

        println!("✓ Performance measurement completed for '{}': {:?}", self.operation, result.duration);
        result
    }
}

/// Measure frame rate over a period of time
pub fn measure_frame_rate(duration: Duration) -> Result<f32, String> {
    let start_time = Instant::now();
    let mut frame_count = 0;

    while start_time.elapsed() < duration {
        // Simulate frame rendering
        std::thread::sleep(Duration::from_millis(16)); // ~60 FPS
        frame_count += 1;
    }

    let elapsed = start_time.elapsed();
    let fps = frame_count as f32 / elapsed.as_secs_f32();

    println!("📊 Measured frame rate: {:.2} FPS over {:?}", fps, elapsed);
    Ok(fps)
}

/// Measure GUI rendering performance
pub fn measure_rendering_performance(operation: &str, iterations: usize) -> Result<PerformanceResult, String> {
    let mut total_duration = Duration::default();

    for i in 0..iterations {
        let timer = PerformanceTimer::start(&format!("{}_iteration_{}", operation, i));
        // Simulate rendering operation
        std::thread::sleep(Duration::from_micros(100));
        let result = timer.stop();
        total_duration += result.duration;
    }

    let avg_duration = total_duration / iterations as u32;

    let mut result = PerformanceResult::new(operation);
    result.duration = avg_duration;
    result.metadata.insert("iterations".to_string(), iterations.to_string());
    result.metadata.insert("total_duration".to_string(), format!("{:?}", total_duration));

    println!("📈 Average rendering time for '{}': {:?} ({} iterations)",
        operation, avg_duration, iterations);

    Ok(result)
}

/// Measure memory usage (placeholder implementation)
pub fn measure_memory_usage() -> Result<HashMap<String, u64>, String> {
    // Placeholder - in real implementation, this would use system APIs
    let mut memory_stats = HashMap::new();
    memory_stats.insert("rss_bytes".to_string(), 1024 * 1024 * 50); // 50 MB placeholder
    memory_stats.insert("virtual_bytes".to_string(), 1024 * 1024 * 100); // 100 MB placeholder

    println!("🧠 Memory usage - RSS: {} bytes, Virtual: {} bytes",
        memory_stats["rss_bytes"], memory_stats["virtual_bytes"]);

    Ok(memory_stats)
}

/// Performance benchmark for GUI operations
pub struct PerformanceBenchmark {
    results: Vec<PerformanceResult>,
    name: String,
}

impl PerformanceBenchmark {
    pub fn new(name: &str) -> Self {
        Self {
            results: Vec::new(),
            name: name.to_string(),
        }
    }

    pub fn measure<F>(&mut self, operation_name: &str, operation: F) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), String>,
    {
        let timer = PerformanceTimer::start(operation_name);
        operation()?;
        let result = timer.stop();
        self.results.push(result);
        Ok(())
    }

    pub fn get_results(&self) -> &[PerformanceResult] {
        &self.results
    }

    pub fn summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        if self.results.is_empty() {
            summary.insert("status".to_string(), "No measurements recorded".to_string());
            return summary;
        }

        let total_duration: Duration = self.results.iter().map(|r| r.duration).sum();
        let avg_duration = total_duration / self.results.len() as u32;
        let min_duration = self.results.iter().map(|r| r.duration).min().unwrap();
        let max_duration = self.results.iter().map(|r| r.duration).max().unwrap();

        summary.insert("benchmark_name".to_string(), self.name.clone());
        summary.insert("total_operations".to_string(), self.results.len().to_string());
        summary.insert("total_duration".to_string(), format!("{:?}", total_duration));
        summary.insert("average_duration".to_string(), format!("{:?}", avg_duration));
        summary.insert("min_duration".to_string(), format!("{:?}", min_duration));
        summary.insert("max_duration".to_string(), format!("{:?}", max_duration));

        summary
    }

    pub fn print_summary(&self) {
        let summary = self.summary();
        println!("📊 Performance Benchmark Summary: {}", self.name);
        for (key, value) in &summary {
            println!("  {}: {}", key, value);
        }
    }
}

/// Check if performance meets requirements
pub fn check_performance_threshold(result: &PerformanceResult, max_duration: Duration) -> Result<(), String> {
    if result.duration > max_duration {
        Err(format!(
            "Performance threshold exceeded for '{}': {:?} > {:?}",
            result.operation, result.duration, max_duration
        ))
    } else {
        println!("✓ Performance check passed for '{}': {:?}", result.operation, result.duration);
        Ok(())
    }
}

/// Performance regression detector
pub struct PerformanceRegressionDetector {
    baseline_results: HashMap<String, Duration>,
    threshold_percent: f32,
}

impl PerformanceRegressionDetector {
    pub fn new(threshold_percent: f32) -> Self {
        Self {
            baseline_results: HashMap::new(),
            threshold_percent,
        }
    }

    pub fn set_baseline(&mut self, operation: &str, duration: Duration) {
        self.baseline_results.insert(operation.to_string(), duration);
        println!("📋 Set baseline for '{}': {:?}", operation, duration);
    }

    pub fn check_regression(&self, result: &PerformanceResult) -> Result<(), String> {
        if let Some(&baseline) = self.baseline_results.get(&result.operation) {
            let regression_percent = (result.duration.as_secs_f64() - baseline.as_secs_f64()) / baseline.as_secs_f64() * 100.0;

            if regression_percent > self.threshold_percent as f64 {
                return Err(format!(
                    "Performance regression detected for '{}': {:.2}% slower than baseline ({:?} vs {:?})",
                    result.operation, regression_percent, result.duration, baseline
                ));
            } else {
                println!("✓ No performance regression for '{}': {:.2}% change", result.operation, regression_percent);
            }
        } else {
            println!("⚠️  No baseline set for '{}', skipping regression check", result.operation);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_timer() {
        let timer = PerformanceTimer::start("test_operation");
        std::thread::sleep(Duration::from_millis(10));
        let result = timer.stop();

        assert_eq!(result.operation, "test_operation");
        assert!(result.duration >= Duration::from_millis(10));
    }

    #[test]
    fn test_measure_frame_rate() {
        let result = measure_frame_rate(Duration::from_millis(100));
        assert!(result.is_ok());
        let fps = result.unwrap();
        assert!(fps > 0.0);
    }

    #[test]
    fn test_performance_benchmark() {
        let mut benchmark = PerformanceBenchmark::new("test_benchmark");

        benchmark.measure("operation1", || {
            std::thread::sleep(Duration::from_millis(5));
            Ok(())
        }).unwrap();

        benchmark.measure("operation2", || {
            std::thread::sleep(Duration::from_millis(10));
            Ok(())
        }).unwrap();

        assert_eq!(benchmark.get_results().len(), 2);
        let summary = benchmark.summary();
        assert_eq!(summary["total_operations"], "2");
    }

    #[test]
    fn test_check_performance_threshold() {
        let mut result = PerformanceResult::new("test");
        result.duration = Duration::from_millis(50);

        assert!(check_performance_threshold(&result, Duration::from_millis(100)).is_ok());
        assert!(check_performance_threshold(&result, Duration::from_millis(25)).is_err());
    }

    #[test]
    fn test_performance_regression_detector() {
        let mut detector = PerformanceRegressionDetector::new(10.0);

        detector.set_baseline("test_op", Duration::from_millis(100));

        let mut result = PerformanceResult::new("test_op");
        result.duration = Duration::from_millis(105); // 5% slower

        assert!(detector.check_regression(&result).is_ok());

        result.duration = Duration::from_millis(120); // 20% slower
        assert!(detector.check_regression(&result).is_err());
    }
}