//! Performance benchmarks for Phase 2D: Memory Optimization
//! 
//! Tests memory allocation efficiency, buffer pooling, and throughput performance

#[cfg(test)]
mod tests {
    use tn5250r::telnet_negotiation::{TelnetNegotiator, BufferPool, BufferPoolMetrics};
    use std::time::Instant;

    #[test]
    fn test_buffer_pool_efficiency() {
        let buffer_pool = BufferPool::new();
        
        // Test that we get buffers of appropriate sizes
        let small_buffer = buffer_pool.get_buffer(32);
        assert!(small_buffer.capacity() >= 32);
        assert!(small_buffer.capacity() <= 64);
        
        let medium_buffer = buffer_pool.get_buffer(256);
        assert!(medium_buffer.capacity() >= 256);
        assert!(medium_buffer.capacity() <= 512);
        
        let large_buffer = buffer_pool.get_buffer(2048);
        assert!(large_buffer.capacity() >= 2048);
        assert!(large_buffer.capacity() <= 4096);
        
        // Test buffer reuse
        buffer_pool.return_buffer(small_buffer);
        buffer_pool.return_buffer(medium_buffer);
        buffer_pool.return_buffer(large_buffer);
        
        // Get new buffers - should reuse from pool
        let reused_small = buffer_pool.get_buffer(32);
        let reused_medium = buffer_pool.get_buffer(256);
        let reused_large = buffer_pool.get_buffer(2048);
        
        // Verify metrics show reuse
        let metrics = buffer_pool.get_metrics();
        assert!(metrics.small_reuses > 0);
        assert!(metrics.medium_reuses > 0);
        assert!(metrics.large_reuses > 0);
        
        // Return buffers for cleanup
        buffer_pool.return_buffer(reused_small);
        buffer_pool.return_buffer(reused_medium);
        buffer_pool.return_buffer(reused_large);
    }

    #[test]
    fn test_buffer_pool_metrics_tracking() {
        let buffer_pool = BufferPool::new();
        buffer_pool.reset_metrics();
        
        // Allocate various buffer sizes
        let mut buffers = Vec::new();
        buffers.push(buffer_pool.get_buffer(16));    // Small
        buffers.push(buffer_pool.get_buffer(128));   // Medium
        buffers.push(buffer_pool.get_buffer(1024));  // Large
        buffers.push(buffer_pool.get_buffer(32));    // Small
        buffers.push(buffer_pool.get_buffer(256));   // Medium
        
        let metrics = buffer_pool.get_metrics();
        assert_eq!(metrics.small_allocations, 2);
        assert_eq!(metrics.medium_allocations, 2);
        assert_eq!(metrics.large_allocations, 1);
        assert!(metrics.total_bytes_allocated > 0);
        
        // Return all buffers
        for buffer in buffers {
            buffer_pool.return_buffer(buffer);
        }
        
        // Allocate again - should show reuse
        let _small = buffer_pool.get_buffer(16);
        let _medium = buffer_pool.get_buffer(128);
        let _large = buffer_pool.get_buffer(1024);
        
        let final_metrics = buffer_pool.get_metrics();
        assert!(final_metrics.small_reuses > 0);
        assert!(final_metrics.medium_reuses > 0);
        assert!(final_metrics.large_reuses > 0);
        
        // Efficiency ratio should be meaningful
        let efficiency = final_metrics.get_efficiency_ratio();
        assert!(efficiency > 0.0);
        assert!(efficiency <= 1.0);
    }

    #[test]
    fn test_optimized_negotiation_performance() {
        let mut negotiator = TelnetNegotiator::new();
        negotiator.reset_buffer_pool_metrics();
        
        // Simulate large negotiation sequence
        let negotiation_data = create_large_negotiation_sequence();
        
        let start_time = Instant::now();
        let response = negotiator.process_incoming_data_optimized(&negotiation_data);
        let processing_time = start_time.elapsed();
        
        // Verify performance characteristics
        assert!(!response.is_empty());
        assert!(processing_time.as_millis() < 100); // Should be fast
        
        // Check buffer pool efficiency
        let pool_metrics = negotiator.get_buffer_pool_metrics();
        assert!(pool_metrics.total_bytes_allocated > 0);
        
        // Efficiency should be good for repeated operations
        let efficiency = pool_metrics.get_efficiency_ratio();
        println!("Buffer pool efficiency: {:.2}%", efficiency * 100.0);
    }

    #[test]
    fn test_memory_allocation_patterns() {
        let mut negotiator = TelnetNegotiator::new();
        negotiator.reset_buffer_pool_metrics();
        
        // Test different sized negotiation sequences
        let small_sequence = create_small_negotiation_sequence();
        let medium_sequence = create_medium_negotiation_sequence();
        let large_sequence = create_large_negotiation_sequence();
        
        // Process each sequence
        let _response1 = negotiator.process_incoming_data_optimized(&small_sequence);
        let _response2 = negotiator.process_incoming_data_optimized(&medium_sequence);
        let _response3 = negotiator.process_incoming_data_optimized(&large_sequence);
        
        // Verify appropriate buffer sizes were used
        let metrics = negotiator.get_buffer_pool_metrics();
        assert!(metrics.small_allocations > 0 || metrics.small_reuses > 0, 
                "Expected small buffer usage but got allocations: {}, reuses: {}", 
                metrics.small_allocations, metrics.small_reuses);
        assert!(metrics.medium_allocations > 0 || metrics.medium_reuses > 0,
                "Expected medium buffer usage but got allocations: {}, reuses: {}", 
                metrics.medium_allocations, metrics.medium_reuses);
        assert!(metrics.large_allocations > 0 || metrics.large_reuses > 0,
                "Expected large buffer usage but got allocations: {}, reuses: {}", 
                metrics.large_allocations, metrics.large_reuses);
    }

    #[test]
    fn test_throughput_benchmarking() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Create test data
        let test_sequences = vec![
            create_small_negotiation_sequence(),
            create_medium_negotiation_sequence(),
            create_large_negotiation_sequence(),
        ];
        
        let mut total_bytes_processed = 0;
        
        let benchmark_start = Instant::now();
        
        // Process sequences multiple times to measure throughput
        for _ in 0..100 {
            for sequence in &test_sequences {
                let response = negotiator.process_incoming_data_optimized(sequence);
                total_bytes_processed += sequence.len() + response.len();
            }
        }
        
        let benchmark_duration = benchmark_start.elapsed();
        
        // Calculate throughput metrics
        let bytes_per_second = total_bytes_processed as f64 / benchmark_duration.as_secs_f64();
        let sequences_per_second = 300.0 / benchmark_duration.as_secs_f64(); // 100 iterations * 3 sequences
        
        println!("Throughput: {:.0} bytes/sec, {:.0} sequences/sec", bytes_per_second, sequences_per_second);
        
        // Verify reasonable performance
        assert!(bytes_per_second > 10000.0); // At least 10KB/s
        assert!(sequences_per_second > 100.0); // At least 100 sequences/s
        
        // Check memory efficiency
        let final_metrics = negotiator.get_buffer_pool_metrics();
        let efficiency = final_metrics.get_efficiency_ratio();
        assert!(efficiency > 0.5); // At least 50% buffer reuse
    }

    #[test]
    fn test_concurrent_buffer_usage() {
        use std::sync::Arc;
        use std::thread;
        
        let buffer_pool = Arc::new(BufferPool::new());
        let mut handles = vec![];
        
        // Spawn multiple threads using the same buffer pool
        for thread_id in 0..4 {
            let pool = Arc::clone(&buffer_pool);
            let handle = thread::spawn(move || {
                let mut buffers = Vec::new();
                
                // Each thread allocates and returns buffers
                for i in 0..10 {
                    let size = match (thread_id + i) % 3 {
                        0 => 32,   // Small
                        1 => 256,  // Medium
                        _ => 1024, // Large
                    };
                    buffers.push(pool.get_buffer(size));
                }
                
                // Return all buffers
                for buffer in buffers {
                    pool.return_buffer(buffer);
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify metrics are consistent
        let final_metrics = buffer_pool.get_metrics();
        assert!(final_metrics.total_bytes_allocated > 0);
        let efficiency = final_metrics.get_efficiency_ratio();
        println!("Concurrent buffer pool efficiency: {:.2}%", efficiency * 100.0);
    }

    #[test]
    fn test_negotiation_completion_performance() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Test time to complete full negotiation
        let full_negotiation = create_complete_negotiation_sequence();
        
        let start = Instant::now();
        let response = negotiator.process_incoming_data_optimized(&full_negotiation);
        let completion_time = start.elapsed();
        
        // Verify negotiation completed
        assert!(!response.is_empty());
        assert!(negotiator.is_negotiation_complete());
        
        // Performance should be good
        assert!(completion_time.as_millis() < 50); // Complete negotiation in under 50ms
        
        // Log performance metrics
        println!("Negotiation completion: {}ms", completion_time.as_millis());
    }

    // Helper functions to create test data
    fn create_small_negotiation_sequence() -> Vec<u8> {
        vec![
            255, 251, 0,  // IAC WILL BINARY
            255, 251, 19, // IAC WILL EOR
        ]
    }

    fn create_medium_negotiation_sequence() -> Vec<u8> {
        let mut sequence = Vec::with_capacity(400);
        
        // Create a medium-sized sequence (~300 bytes, should use medium buffer)
        for _ in 0..20 {
            sequence.extend_from_slice(&[255, 251, 0]);  // IAC WILL BINARY
            sequence.extend_from_slice(&[255, 251, 19]); // IAC WILL EOR
            sequence.extend_from_slice(&[255, 251, 3]);  // IAC WILL SGA
            sequence.extend_from_slice(&[255, 253, 24]); // IAC DO TERMINAL-TYPE
            sequence.extend_from_slice(&[255, 253, 39]); // IAC DO NEW-ENVIRON
        }
        
        sequence
    }

    fn create_large_negotiation_sequence() -> Vec<u8> {
        let mut sequence = Vec::with_capacity(5000);
        
        // Add multiple rounds of negotiations to create a large buffer (over 4KB)
        for _ in 0..200 {
            sequence.extend_from_slice(&[255, 251, 0]);  // IAC WILL BINARY
            sequence.extend_from_slice(&[255, 251, 19]); // IAC WILL EOR
            sequence.extend_from_slice(&[255, 251, 3]);  // IAC WILL SGA
            sequence.extend_from_slice(&[255, 253, 24]); // IAC DO TERMINAL-TYPE
            sequence.extend_from_slice(&[255, 253, 39]); // IAC DO NEW-ENVIRON
        }
        
        sequence
    }

    fn create_complete_negotiation_sequence() -> Vec<u8> {
        vec![
            255, 251, 0,  // IAC WILL BINARY
            255, 251, 19, // IAC WILL EOR
            255, 251, 3,  // IAC WILL SGA
            255, 254, 1,  // IAC DONT ECHO
        ]
    }
}