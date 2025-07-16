use mini_redis::{Result, client};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "127.0.0.1:6379";
    let mut client = client::connect(url).await?;
    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;
    println!("Got a value from Redis: {:?}", result);

    let future = client.get("key");
    println!("Doing something else");
    let result = future.await?;
    println!("Got a value from Redis: {:?}", result);

    Ok(())
}
