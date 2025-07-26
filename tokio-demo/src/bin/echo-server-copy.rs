use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        TcpListener, TcpStream,
        tcp::{ReadHalf, WriteHalf},
    },
};

async fn echo_line<'a>(
    input: &'static str,
    description: &'static str,
    writer: &mut WriteHalf<'a>,
    buf_reader: &mut BufReader<ReadHalf<'a>>,
) -> io::Result<()> {
    writer.write_all(input.as_bytes()).await?;
    let mut line = String::new();
    buf_reader.read_line(&mut line).await?;
    if let Some(end) = line.pop() {
        if end != '\n' {
            panic!("Expected line to end with newline, but got: {}", end);
        }
        if line.chars().last() == Some('\r') {
            let _ = line.pop();
            println!("# Removed \\r\\n");
        } else {
            println!("# Removed \\n");
        }
        println!("Received {} line: {}", description, line);
    }
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let url = "127.0.0.1:6142";
    let listener = TcpListener::bind(&url).await?;

    let accept_handle = listener.accept();

    let handle = tokio::spawn(async move {
        let mut client = TcpStream::connect(&url).await?;
        let (reader, mut writer) = client.split();
        let mut buf_reader = io::BufReader::new(reader);

        echo_line("Hello, world!\r\n", "1st", &mut writer, &mut buf_reader).await?;
        echo_line("Hello, tokio!\r\n", "2nd", &mut writer, &mut buf_reader).await?;

        // Add the type hint so that we can use .await?
        Ok::<_, io::Error>(())
    });

    let (mut socket, addr) = accept_handle.await?;
    println!("New connection from: {}", addr);
    let (mut reader, mut writer) = socket.split();
    io::copy(&mut reader, &mut writer).await?;

    handle.await??;

    Ok(())
}
