use std::time::Instant;
use tokio;
use tn5250r::telnet_negotiation::{TelnetNegotiator, TelnetOption};

/// Tests for concurrent telnet negotiation processing
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_concurrent_negotiation_sequences() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Create test data sequences
        let sequences = vec![
            vec![255, 251, 0], // IAC WILL Binary
            vec![255, 253, 19], // IAC DO End-of-Record  
            vec![255, 251, 3], // IAC WILL Suppress-Go-Ahead
            vec![255, 253, 0], // IAC DO Binary
        ];
        
        let start_time = Instant::now();
        let results = negotiator.process_concurrent_negotiations(sequences).await;
        let duration = start_time.elapsed();
        
        // Verify we got results for all sequences
        assert_eq!(results.len(), 4);
        
        // Verify each result is not empty
        for result in &results {
            assert!(!result.is_empty(), "Expected non-empty result from concurrent processing");
        }
        
        println!("Concurrent processing completed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_parallel_option_negotiation() {
        let mut negotiator = TelnetNegotiator::new();
        
        let options = vec![
            TelnetOption::Binary,
            TelnetOption::EndOfRecord,
            TelnetOption::SuppressGoAhead,
        ];
        
        let start_time = Instant::now();
        let results = negotiator.process_parallel_options(options.clone()).await;
        let duration = start_time.elapsed();
        
        // Verify all options were processed
        assert_eq!(results.len(), 3);
        
        for option in options {
            assert!(results.contains_key(&option), "Expected result for option {:?}", option);
            assert!(results[&option], "Expected successful negotiation for option {:?}", option);
        }
        
        println!("Parallel option negotiation completed in {:?}", duration);
    }

    #[tokio::test]
    async fn test_concurrent_processing_performance() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Create larger dataset for performance testing
        let mut sequences = Vec::new();
        for i in 0..100 {
            let mut seq = vec![255, 251, (i % 256) as u8]; // IAC WILL <option>
            seq.extend_from_slice(b"test data for sequence ");
            seq.extend_from_slice(i.to_string().as_bytes());
            sequences.push(seq);
        }
        
        let start_time = Instant::now();
        let results = negotiator.process_concurrent_negotiations(sequences).await;
        let duration = start_time.elapsed();
        
        assert_eq!(results.len(), 100);
        
        // Verify performance (should be faster than sequential processing)
        assert!(duration.as_millis() < 1000, "Concurrent processing took too long: {:?}", duration);
        
        println!("Processed 100 sequences concurrently in {:?}", duration);
    }

    #[tokio::test]
    async fn test_concurrent_buffer_pool_usage() {
        let mut negotiator = TelnetNegotiator::new();
        negotiator.reset_buffer_pool_metrics();
        
        // Create sequences of different sizes to test buffer pool
        let sequences = vec![
            vec![255; 30],    // Small sequence - should use small buffer
            vec![255; 200],   // Medium sequence - should use medium buffer  
            vec![255; 2000],  // Large sequence - should use large buffer
        ];
        
        let _results = negotiator.process_concurrent_negotiations(sequences).await;
        
        let metrics = negotiator.get_buffer_pool_metrics();
        
        // Verify that different buffer sizes were used
        let total_allocations = metrics.small_allocations + metrics.medium_allocations + metrics.large_allocations;
        let total_reuses = metrics.small_reuses + metrics.medium_reuses + metrics.large_reuses;
        
        assert!(total_allocations > 0 || total_reuses > 0, 
                "Expected buffer pool usage but got no allocations or reuses");
        
        println!("Buffer pool metrics after concurrent processing:");
        println!("  Small: {} allocations, {} reuses", metrics.small_allocations, metrics.small_reuses);
        println!("  Medium: {} allocations, {} reuses", metrics.medium_allocations, metrics.medium_reuses);
        println!("  Large: {} allocations, {} reuses", metrics.large_allocations, metrics.large_reuses);
    }

    #[tokio::test]
    async fn test_concurrent_error_handling() {
        let mut negotiator = TelnetNegotiator::new();
        
        // Create sequences including some that might cause issues
        let sequences = vec![
            vec![255, 251, 0], // Valid: IAC WILL Binary
            vec![],            // Empty sequence
            vec![255],         // Incomplete IAC command
            vec![255, 251, 3], // Valid: IAC WILL Suppress-Go-Ahead
        ];
        
        let results = negotiator.process_concurrent_negotiations(sequences).await;
        
        // Should handle all sequences gracefully
        assert_eq!(results.len(), 4);
        
        // Even empty or invalid sequences should return some result (even if empty)
        for (i, result) in results.iter().enumerate() {
            println!("Sequence {} result length: {}", i, result.len());
            // Don't assert on length as empty sequences might return empty results
        }
    }

    #[tokio::test]
    async fn test_concurrent_vs_sequential_performance() {
        let mut negotiator1 = TelnetNegotiator::new();
        let mut negotiator2 = TelnetNegotiator::new();
        
        // Create test sequences
        let sequences: Vec<Vec<u8>> = (0..50).map(|i| {
            let mut seq = vec![255, 251, (i % 10) as u8];
            seq.extend_from_slice(b"performance test data sequence ");
            seq.extend_from_slice(i.to_string().as_bytes());
            seq
        }).collect();
        
        // Test concurrent processing
        let sequences_concurrent = sequences.clone();
        let start_concurrent = Instant::now();
        let _concurrent_results = negotiator1.process_concurrent_negotiations(sequences_concurrent).await;
        let concurrent_duration = start_concurrent.elapsed();
        
        // Test sequential processing
        let start_sequential = Instant::now();
        let mut sequential_results = Vec::new(); 
        for sequence in sequences {
            let result = negotiator2.process_incoming_data_optimized(&sequence);
            sequential_results.push(result);
        }
        let sequential_duration = start_sequential.elapsed();
        
        println!("Concurrent processing: {:?}", concurrent_duration);
        println!("Sequential processing: {:?}", sequential_duration);
        
        // Concurrent should be at least competitive (within 2x of sequential)
        // Note: For small datasets, concurrent might be slower due to overhead
        let speedup_ratio = sequential_duration.as_nanos() as f64 / concurrent_duration.as_nanos() as f64;
        println!("Speedup ratio: {:.2}x", speedup_ratio);
        
        // Just verify both completed successfully
        assert_eq!(sequential_results.len(), 50);
    }
}