use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let url = "127.0.0.1:6142";
    let listener = TcpListener::bind(&url).await?;

    let accept_handle = listener.accept();

    let handle = tokio::spawn(async move {
        let mut client = TcpStream::connect(&url).await?;
        let (reader, mut writer) = client.split();
        let mut buf_reader = io::BufReader::new(reader);

        let mut echo_line = async |input: &'static str, description: &'static str| {
            writer.write_all(input.as_bytes()).await?;
            let mut line = String::new();
            buf_reader.read_line(&mut line).await?;
            if let Some('\n') = line.pop() {
                if let Some('\r') = line.chars().last() {
                    let _ = line.pop();
                    println!("# Removed \\r\\n");
                } else {
                    println!("# Removed \\n");
                }
                println!("Received {} line: {}", description, line);
            } else {
                panic!("Expected line to end with newline, but got {}", line);
            }
            Ok::<_, io::Error>(())
        };

        echo_line("Hello, world!\r\n", "1st").await?;
        echo_line("Hello, tokio!\n", "2nd").await?;

        // Add the type hint so that we can use .await?
        Ok::<_, io::Error>(())
    });

    // The following logic is basically same with `io::copy` but with custom logs.
    let (mut socket, addr) = accept_handle.await?;
    println!("New connection from: {}", addr);
    let mut buf = vec![0; 1024];
    loop {
        match socket.read(&mut buf).await {
            Ok(0) => {
                println!("Connection closed by {}", addr);
                break;
            }
            Ok(n) => {
                if socket.write_all(&buf[..n]).await.is_err() {
                    println!("Failed to write to {}", addr);
                    break;
                }
            }
            Err(err) => {
                println!("Error reading from {}: {}", addr, err);
                break;
            }
        }
    }

    handle.await??;

    Ok(())
}
