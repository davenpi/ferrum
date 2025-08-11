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
    println!("🚀 Starting Ferrum task example");

    // Create a scheduler
    let scheduler = LocalScheduler::new();

    // Create a distributed task using our macro
    let task = ferrum_task!(|| async { expensive_computation(42).await });

    println!("📤 Submitting task...");

    // Submit and wait for result
    let result = scheduler.submit(task).await?;

    println!("✅ Task completed! Result: {}", result);

    // Let's try multiple tasks
    println!("\n🔄 Running multiple tasks...");

    // Create tasks individually and submit them
    let values = vec![1, 2, 3];

    for (i, value) in values.into_iter().enumerate() {
        let task = ferrum_task!(move || async move { expensive_computation(value).await });
        let result = scheduler.submit(task).await?;
        println!("Task {}: {}", i + 1, result);
    }

    println!("\n🎉 All tasks completed!");
    Ok(())
}
