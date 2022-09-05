pub mod util;
pub mod mc;

use crate::util::ByteRead;
use crate::util::McRead;
use crate::util::Timer;
use mc_varint::VarIntRead;
use tokio::io::AsyncReadExt;
use tokio::io::copy_bidirectional;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use std::error::Error;
use std::io::Cursor;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Debug)]
pub struct Args {
    pub bind_address: SocketAddr,
    pub target_address: SocketAddr,
}
impl Args {
    pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, Box<dyn Error>> {
        args.next(); // skip argv[0] (binary path)

        let bind_address = args.next().ok_or("missing bind address")?;
        let bind_address = bind_address.parse()?;

        let target_address = args.next().ok_or("missing target address")?;
        let target_address = target_address.parse()?;

        Ok(Args { bind_address, target_address })
    }
}

pub async fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(args.bind_address).await?;

    eprintln!("Listening... (bound to {})", args.bind_address);

    let time = Arc::new(Mutex::new(Instant::now()));
    let args = Arc::new(args);

    loop {
        let (stream, _) = listener.accept().await?;

        if let Ok(mut time) = time.lock() {
            eprintln!( "\n[+{:.6}] {:?}", time.get_elapsed_then_reset().as_secs_f32(), stream);
        }

        let time = Arc::clone(&time);
        let args = Arc::clone(&args);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(&args, stream).await {
                eprintln!("Error while handling connection: {}", e);
            }
            if let Ok(mut time) = time.lock() {
                eprintln!("[+{:.6}] ;", time.get_elapsed_then_reset().as_secs_f32());
            }
        });
    }
}

async fn handle_connection(args: &Args, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let target = TcpStream::connect(args.target_address);

    let mut first_byte = [0u8];
    // TODO: there must be other way around than .peek() ; maybe .bytes()
    stream
        .peek(&mut first_byte)
        .await
        .map_err(|e| format!("peek() failed: {e}"))?;

    if first_byte[0] == 0xFE {
        println!("Kind?: Legacy Handshake");

        let mut buf = Vec::with_capacity(256);
        stream
            .read_to_end(&mut buf)
            .await
            .map_err(|e| format!("read_to_end() failed: {e}"))?;

        eprintln!("len={}, {:?}", buf.len(), buf);
        let mut cur = Cursor::new(buf);
        println!("{:X}", cur.read_byte().unwrap());
        println!("{:X}", cur.read_byte().unwrap());

        return Ok(());
    }

    println!("Kind?: Modern Packet");

    let packet_length: usize = i32::from(stream.read_var_int()?).try_into().unwrap();
    println!("packet_length = {packet_length}");

    let buffer = {
        let mut buffer = vec![0u8; packet_length];
        stream.read_exact(&mut buffer).await?;
        buffer
    };
    let mut cur = Cursor::new(&buffer);

    let packet_id: i32 = cur.read_var_int()?.into();
    println!("packet_id = {packet_id}");

    let protocol_version: i32 = cur.read_var_int()?.into();
    println!("protocol_version = {protocol_version}");

    let names = cur.read_mc_string()?;
    let mut names = names.split_inclusive(|b| *b == 0);

    let hostname = names.next()
        .map(|name| String::from_utf8_lossy(name.split_last().unwrap().1));
    println!("hostname = {hostname:?}");

    while let Some(custom_name) = names.next() {
        let custom_name = String::from_utf8_lossy(custom_name);
        println!("custom_name = {custom_name}");
    }

    let port = cur.read_u16().await?;
    println!("port = {port}");

    println!("? buffer = {buffer:?}");

    let target = target.await?;
    let copy = copy_bidirectional(&mut stream, &mut target);

    Ok(())
}
