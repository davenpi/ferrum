use ferrum::task;

#[task]
async fn simple_computation(x: i32) -> i32 {
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    x * x
}

#[task]
fn simple_computation_2(x: i32) -> i32 {
    x * x
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Ferrum attribute macro");
    
    let result = simple_computation(7).await?;
    println!("7² = {}", result);

    let handle = simple_computation_2(7);
    println!("7² = {}", handle.await?);

    Ok(())
}
