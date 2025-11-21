//! Performance and load tests for ZIK_ZAK
//!
//! Tests the system under various load conditions to ensure
//! it can handle the claimed performance characteristics.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use zik_zak::ZikZakEngine;

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_transfer_throughput() -> Result<()> {
    println!("🚀 Testing transfer throughput");

    let engine = Arc::new(Mutex::new(ZikZakEngine::new().await?));

    // Pre-fund accounts
    {
        let mut eng = engine.lock().await;
        for i in 0..100 {
            eng.transfer(
                "system:genesis",
                &format!("user:{}:balance", i),
                1_000_000,
                HashMap::new(),
            )
            .await?;
        }
    }

    // Measure throughput
    let num_transfers = 1000;
    let start = Instant::now();

    for i in 0..num_transfers {
        let from = i % 100;
        let to = (i + 1) % 100;

        let mut eng = engine.lock().await;
        eng.transfer(
            &format!("user:{}:balance", from),
            &format!("user:{}:balance", to),
            100,
            HashMap::new(),
        )
        .await?;
    }

    let duration = start.elapsed();
    let tps = num_transfers as f64 / duration.as_secs_f64();

    println!("  ✅ Completed {} transfers in {:?}", num_transfers, duration);
    println!("  📊 Throughput: {:.2} transfers/second", tps);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_balance_query_performance() -> Result<()> {
    println!("🚀 Testing balance query performance");

    let engine = ZikZakEngine::new().await?;

    // Create some accounts
    for i in 0..100 {
        engine
            .transfer(
                "system:genesis",
                &format!("test:{}:balance", i),
                1000,
                HashMap::new(),
            )
            .await?;
    }

    // Measure query performance
    let num_queries = 10_000;
    let start = Instant::now();

    for i in 0..num_queries {
        let account = format!("test:{}:balance", i % 100);
        let _balance = engine.get_balance(&account).await?;
    }

    let duration = start.elapsed();
    let qps = num_queries as f64 / duration.as_secs_f64();

    println!("  ✅ Completed {} queries in {:?}", num_queries, duration);
    println!("  📊 Query rate: {:.2} queries/second", qps);

    if qps > 10_000.0 {
        println!("  🎯 Excellent! > 10k queries/second");
    } else if qps > 5_000.0 {
        println!("  ✅ Good! > 5k queries/second");
    } else {
        println!("  ⚠️  Consider optimization (< 5k queries/second)");
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_concurrent_transfers() -> Result<()> {
    println!("🚀 Testing concurrent transfer performance");

    let engine = Arc::new(Mutex::new(ZikZakEngine::new().await?));

    // Pre-fund accounts
    {
        let mut eng = engine.lock().await;
        for i in 0..1000 {
            eng.transfer(
                "system:genesis",
                &format!("user:{}:balance", i),
                100_000,
                HashMap::new(),
            )
            .await?;
        }
    }

    // Run concurrent transfers
    let num_workers = 10;
    let transfers_per_worker = 100;

    let start = Instant::now();
    let mut handles = vec![];

    for worker_id in 0..num_workers {
        let engine_clone = Arc::clone(&engine);

        let handle = tokio::spawn(async move {
            for i in 0..transfers_per_worker {
                let from = (worker_id * transfers_per_worker + i) % 1000;
                let to = (from + 1) % 1000;

                let mut eng = engine_clone.lock().await;
                eng.transfer(
                    &format!("user:{}:balance", from),
                    &format!("user:{}:balance", to),
                    10,
                    HashMap::new(),
                )
                .await
                .unwrap();
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }

    let duration = start.elapsed();
    let total_transfers = num_workers * transfers_per_worker;
    let tps = total_transfers as f64 / duration.as_secs_f64();

    println!(
        "  ✅ {} workers completed {} transfers in {:?}",
        num_workers, total_transfers, duration
    );
    println!("  📊 Concurrent throughput: {:.2} transfers/second", tps);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_account_creation_performance() -> Result<()> {
    println!("🚀 Testing account creation performance");

    let engine = ZikZakEngine::new().await?;
    let num_accounts = 1000;

    let start = Instant::now();

    for i in 0..num_accounts {
        engine
            .transfer(
                "system:genesis",
                &format!("product:{}:existence", i),
                1,
                HashMap::new(),
            )
            .await?;

        engine
            .transfer(
                "system:genesis",
                &format!("product:{}:price", i),
                (i as i64) * 100,
                HashMap::new(),
            )
            .await?;
    }

    let duration = start.elapsed();
    let accounts_per_sec = (num_accounts * 2) as f64 / duration.as_secs_f64();

    println!(
        "  ✅ Created {} products ({} accounts) in {:?}",
        num_accounts,
        num_accounts * 2,
        duration
    );
    println!("  📊 Account creation rate: {:.2}/second", accounts_per_sec);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_large_transfer_batch() -> Result<()> {
    println!("🚀 Testing large transfer batch");

    let engine = Arc::new(Mutex::new(ZikZakEngine::new().await?));
    let batch_size = 100;

    // Pre-fund
    {
        let mut eng = engine.lock().await;
        eng.transfer(
            "system:genesis",
            "batch:source:balance",
            1_000_000,
            HashMap::new(),
        )
        .await?;
    }

    let start = Instant::now();

    // Execute batch
    for i in 0..batch_size {
        let mut eng = engine.lock().await;
        eng.transfer(
            "batch:source:balance",
            &format!("batch:dest:{}:balance", i),
            100,
            HashMap::new(),
        )
        .await?;
    }

    let duration = start.elapsed();

    println!("  ✅ Batch of {} transfers completed in {:?}", batch_size, duration);
    println!("  📊 Average latency: {:?} per transfer", duration / batch_size);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_memory_usage_under_load() -> Result<()> {
    println!("🚀 Testing memory usage under load");

    let engine = ZikZakEngine::new().await?;

    // Create many accounts and transfers
    let num_entities = 10_000;

    println!("  📝 Creating {} entities...", num_entities);
    let start = Instant::now();

    for i in 0..num_entities {
        engine
            .transfer(
                "system:genesis",
                &format!("entity:{}:existence", i),
                1,
                HashMap::new(),
            )
            .await?;

        if i % 1000 == 0 {
            println!("    Progress: {}/{}", i, num_entities);
        }
    }

    let duration = start.elapsed();

    println!("  ✅ Created {} entities in {:?}", num_entities, duration);
    println!("  💾 Memory test completed (check system monitor for actual usage)");

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for performance testing
async fn test_latency_distribution() -> Result<()> {
    println!("🚀 Testing transfer latency distribution");

    let engine = ZikZakEngine::new().await?;

    // Pre-fund accounts
    for i in 0..10 {
        engine
            .transfer(
                "system:genesis",
                &format!("latency_test:{}:balance", i),
                1_000_000,
                HashMap::new(),
            )
            .await?;
    }

    // Measure individual transfer latencies
    let num_samples = 100;
    let mut latencies = Vec::new();

    for i in 0..num_samples {
        let start = Instant::now();

        engine
            .transfer(
                &format!("latency_test:{}:balance", i % 10),
                &format!("latency_test:{}:balance", (i + 1) % 10),
                100,
                HashMap::new(),
            )
            .await?;

        latencies.push(start.elapsed());
    }

    // Calculate percentiles
    latencies.sort();
    let p50 = latencies[num_samples / 2];
    let p95 = latencies[(num_samples * 95) / 100];
    let p99 = latencies[(num_samples * 99) / 100];
    let max = latencies[num_samples - 1];

    println!("  📊 Latency Distribution:");
    println!("    p50: {:?}", p50);
    println!("    p95: {:?}", p95);
    println!("    p99: {:?}", p99);
    println!("    max: {:?}", max);

    if p99 < Duration::from_millis(10) {
        println!("  🎯 Excellent latency! p99 < 10ms");
    } else if p99 < Duration::from_millis(50) {
        println!("  ✅ Good latency! p99 < 50ms");
    } else {
        println!("  ⚠️  High latency - consider optimization");
    }

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for stress testing
async fn test_sustained_load() -> Result<()> {
    println!("🚀 Testing sustained load (30 seconds)");

    let engine = Arc::new(Mutex::new(ZikZakEngine::new().await?));

    // Pre-fund accounts
    {
        let mut eng = engine.lock().await;
        for i in 0..100 {
            eng.transfer(
                "system:genesis",
                &format!("sustained:{}:balance", i),
                1_000_000_000,
                HashMap::new(),
            )
            .await?;
        }
    }

    let duration = Duration::from_secs(30);
    let start = Instant::now();
    let mut transfer_count = 0u64;

    println!("  ⏱️  Running for 30 seconds...");

    while start.elapsed() < duration {
        let from = transfer_count % 100;
        let to = (transfer_count + 1) % 100;

        let mut eng = engine.lock().await;
        eng.transfer(
            &format!("sustained:{}:balance", from),
            &format!("sustained:{}:balance", to),
            100,
            HashMap::new(),
        )
        .await?;

        transfer_count += 1;

        if transfer_count % 1000 == 0 {
            println!("    {} transfers completed...", transfer_count);
        }
    }

    let actual_duration = start.elapsed();
    let avg_tps = transfer_count as f64 / actual_duration.as_secs_f64();

    println!("  ✅ Sustained load test completed");
    println!("  📊 Total transfers: {}", transfer_count);
    println!("  📊 Average TPS: {:.2}", avg_tps);

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually for benchmarking
async fn test_comparison_baseline() -> Result<()> {
    println!("🚀 Establishing performance baseline");

    let engine = ZikZakEngine::new().await?;

    // Baseline metrics
    println!("\n  📊 BASELINE METRICS:");

    // 1. Single transfer latency
    engine
        .transfer(
            "system:genesis",
            "baseline:warm:up",
            1000,
            HashMap::new(),
        )
        .await?;

    let start = Instant::now();
    engine
        .transfer(
            "system:genesis",
            "baseline:test:balance",
            1000,
            HashMap::new(),
        )
        .await?;
    let single_transfer = start.elapsed();
    println!("    Single transfer: {:?}", single_transfer);

    // 2. Balance query latency
    let start = Instant::now();
    let _balance = engine.get_balance("baseline:test:balance").await?;
    let single_query = start.elapsed();
    println!("    Single query: {:?}", single_query);

    // 3. 100 sequential transfers
    let start = Instant::now();
    for i in 0..100 {
        engine
            .transfer(
                "system:genesis",
                &format!("baseline:{}:test", i),
                100,
                HashMap::new(),
            )
            .await?;
    }
    let batch_100 = start.elapsed();
    println!("    100 transfers: {:?} ({:?}/each)", batch_100, batch_100 / 100);

    println!("\n  🎯 Baseline established for comparison");

    Ok(())
}
