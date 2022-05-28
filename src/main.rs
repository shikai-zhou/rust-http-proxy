use tokio::net::{TcpListener, TcpStream};
use tokio::io::AsyncWriteExt;
use tokio::io;
use std::env;
use std::str;

async fn process_socket(mut inbound: TcpStream) -> io::Result<()>{
    // do work with socket here
    inbound.readable().await?;
    let mut buf = [0; 256];
    inbound.try_read(&mut buf).unwrap();
    let string = str::from_utf8(& buf).unwrap();
    // println!("{}", string);
    let v: Vec<&str> = string.split(' ').collect();
    // println!("{}", v[1]);
    let mut outbound;
    if v[0] == "CONNECT" {
        let uri = parse_uri(&v).await?;
        outbound = TcpStream::connect(uri).await?;
        let mut payload = v[2].to_string();
        payload.push_str(&"200 Connection established\r\n\r\n".to_string());
        inbound.try_write(payload.as_bytes()).unwrap();
    } else {
        //catch all for other requests like GET POST etc
        let uri = parse_uri(&v).await?;
        outbound = TcpStream::connect(uri).await?;
        outbound.try_write(&buf).unwrap();
    }
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}

async fn parse_uri(v : &Vec<&str>) -> io::Result<String>{
    //get uri from request
    //TODO: Check that request is valid
    let uri : Vec<&str> = v[1].split("://").collect();
    let mut hostname: String;
    if uri.len() > 1 {
        let temp : Vec<&str> = uri[1].split('/').collect();
        hostname = temp[0].to_string();
        hostname.push_str(":80");
    } else {
        hostname = uri[0].to_string();
    }
    println!("{}",hostname);
    Ok(hostname)
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut server_addr = "0.0.0.0".to_string();
    if args.len() < 2 {
        server_addr.push_str(":8080");
    } else {
        server_addr.push_str(":");
        server_addr.push_str(&args[1]);
    }
    let listener = TcpListener::bind(server_addr).await?;

    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn( async move {
            
            match process_socket(socket).await {
                Err(e) => println!("Connection Error! {}", e),
                _ => ()
            };
        });
        
    }

}