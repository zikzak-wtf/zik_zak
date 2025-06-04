use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use zik_zak::tigerbeetle_client::TigerBeetleClient;

#[tokio::test]
async fn test_id_uniqueness_guarantees() {
    println!("🧪 Testing ID uniqueness guarantees with fastrand");

    let client = TigerBeetleClient::new().await.expect("Failed to create client");
    
    // Test 1: Single-threaded uniqueness
    let mut ids = HashSet::new();
    let iterations = 100_000;
    
    println!("📊 Testing {} single-threaded IDs...", iterations);
    for i in 0..iterations {
        let id = match i % 4 {
            0 => client.generate_time_based_id(),
            1 => client.generate_random_id(),
            2 => client.generate_client_unique_id(),
            3 => client.generate_machine_unique_id(),
            _ => unreachable!(),
        };
        
        assert!(ids.insert(id), "Duplicate ID found: {}", id);
        
        if i % 10_000 == 0 {
            println!("✅ Generated {} unique IDs so far", i + 1);
        }
    }
    
    println!("🎯 All {} single-threaded IDs are unique!", iterations);
    
    // Test 2: Multi-threaded uniqueness
    let shared_ids = Arc::new(Mutex::new(HashSet::new()));
    let threads = 8;
    let ids_per_thread = 10_000;
    
    println!("🔄 Testing {} threads × {} IDs = {} total IDs...", 
             threads, ids_per_thread, threads * ids_per_thread);
    
    let handles: Vec<_> = (0..threads).map(|thread_id| {
        let client = client.clone(); // Assuming we implement Clone
        let shared_ids = Arc::clone(&shared_ids);
        
        thread::spawn(move || {
            let mut local_ids = Vec::new();
            
            for i in 0..ids_per_thread {
                let id = match i % 4 {
                    0 => client.generate_time_based_id(),
                    1 => client.generate_random_id(), 
                    2 => client.generate_client_unique_id(),
                    3 => client.generate_machine_unique_id(),
                    _ => unreachable!(),
                };
                local_ids.push(id);
            }
            
            // Add all local IDs to shared set
            let mut global_ids = shared_ids.lock().unwrap();
            for id in local_ids {
                assert!(global_ids.insert(id), 
                       "Thread {} found duplicate ID: {}", thread_id, id);
            }
            
            println!("✅ Thread {} completed with {} unique IDs", thread_id, ids_per_thread);
        })
    }).collect();
    
    // Wait for all threads
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    let final_count = shared_ids.lock().unwrap().len();
    println!("🎯 All {} multi-threaded IDs are unique!", final_count);
    assert_eq!(final_count, threads * ids_per_thread);
    
    // Test 3: Transfer ID collision resistance
    println!("🔄 Testing transfer ID collision resistance...");
    let mut transfer_ids = HashSet::new();
    
    for i in 0..10_000 {
        let from_account = client.hash_account_name(&format!("account_{}", i % 100));
        let to_account = client.hash_account_name(&format!("account_{}", (i + 1) % 100));
        
        let transfer_id = client.generate_transfer_id(from_account, to_account);
        assert!(transfer_ids.insert(transfer_id), 
               "Duplicate transfer ID: {}", transfer_id);
    }
    
    println!("🎯 All {} transfer IDs are unique!", transfer_ids.len());
}

#[tokio::test]
async fn test_id_entropy_distribution() {
    println!("🧪 Testing ID entropy and distribution");
    
    let client = TigerBeetleClient::new().await.expect("Failed to create client");
    let sample_size = 10_000;
    
    // Test bit distribution
    let mut bit_counts = [0u32; 128];
    
    for _ in 0..sample_size {
        let id = client.generate_random_id();
        
        for bit_pos in 0..128 {
            if (id >> bit_pos) & 1 == 1 {
                bit_counts[bit_pos] += 1;
            }
        }
    }
    
    println!("📊 Bit distribution analysis (expecting ~50% for each bit):");
    for (bit_pos, count) in bit_counts.iter().enumerate() {
        let percentage = (*count as f64 / sample_size as f64) * 100.0;
        
        // Each bit should be roughly 50% (±5% tolerance for randomness)
        assert!(percentage >= 45.0 && percentage <= 55.0, 
               "Bit {} has poor distribution: {:.2}%", bit_pos, percentage);
        
        if bit_pos % 16 == 0 {
            println!("Bits {}-{}: {:.1}% avg", 
                   bit_pos, (bit_pos + 15).min(127),
                   bit_counts[bit_pos..((bit_pos + 16).min(128))]
                       .iter().map(|&c| c as f64 / sample_size as f64 * 100.0)
                       .sum::<f64>() / 16.0);
        }
    }
    
    println!("🎯 Entropy distribution looks good!");
}

#[tokio::test] 
async fn test_performance_benchmarks() {
    println!("🧪 Performance benchmarks for ID generation");
    
    let client = TigerBeetleClient::new().await.expect("Failed to create client");
    let iterations = 1_000_000;
    
    // Benchmark each ID generation method
    let methods = [
        ("time_based_id", || client.generate_time_based_id()),
        ("random_id", || client.generate_random_id()),
        ("client_unique_id", || client.generate_client_unique_id()),
        ("machine_unique_id", || client.generate_machine_unique_id()),
    ];
    
    for (name, method) in methods.iter() {
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let _ = method();
        }
        
        let duration = start.elapsed();
        let ns_per_id = duration.as_nanos() / iterations as u128;
        let ids_per_sec = 1_000_000_000 / ns_per_id;
        
        println!("⚡ {}: {}ns/ID, {}/s", name, ns_per_id, ids_per_sec);
    }
    
    println!("🎯 Performance benchmarks complete!");
}