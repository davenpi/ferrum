use ferrum::runtime::*;
use tokio;

// A simple computation function
async fn expensive_computation(x: i32) -> i32 {
    // Simulate some work
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    x * x
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Ferrum handle example");
    
    let scheduler = LocalScheduler::new();
    
    // Submit returns a handle
    let handle = scheduler.submit(ferrum_task!(|| async {
        expensive_computation(42).await
    }));
    
    println!("📤 Task submitted with ID: {}", handle.id());
    
    // Await when ready
    let result = handle.await?;
    println!("✅ Task completed! Result: {}", result);
    
    // Batch example
    println!("\n🔄 Running batch tasks...");
    let handles: Vec<_> = (1..=3)
        .map(|i| scheduler.submit(ferrum_task!(move || async move {
            expensive_computation(i).await
        })))
        .collect();
    
    let results = futures::future::join_all(handles).await;
    for (i, result) in results.into_iter().enumerate() {
        println!("Task {}: {:?}", i + 1, result);
    }
    
    Ok(())
}
