use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut f = File::create("foo.txt").await.unwrap();
    f.write_all("hello world你好".as_bytes()).await.unwrap();

    let mut f = File::open("foo.txt").await?;
    let mut buffer = [0; 20];

    // read up to 10 bytes
    let n = f.read(&mut buffer[..]).await?;

    let s = String::from_utf8_lossy(&buffer[..n]);
    println!("The bytes: {:?}", s);
    Ok(())
}
