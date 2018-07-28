extern crate chrono;

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::fs::File;
use std::io;
use std::io::Read;
use std::str;

use chrono::prelude::*;

/**
 * To use:
 *  Place a file in /var/www/  (eg myfile.html)
 *  curl 127.0.0.1:8080/myfile -i
 **/

const PUT_HTML_HERE: &str = "/var/www/";

const TEXT_404: &str = "
<html>
<head><title>404 Not Found</title></head>
<body bgcolor='white'>
<center><h1>404 Not Found</h1></center>
</body>
</html>";


fn create_filename(s: String) -> String {
    let part :String = s[1..].to_string();

    if part.find('.').is_none() {
        part + ".html"
    }
    else {
        part
    }
}

//fn load_file(filename: String) -> Result<u8; 10000], String> {

fn load_file(filename: String) -> Result<File, String> {
    // People could pass in '../../rootfile' in here
    match File::open(format!("{}{}", PUT_HTML_HERE, filename)) {
        Err(why) => return Err(format!("no {}", why).to_string()),
        Ok(file) => return Ok(file),
    }
}

fn read_stream<'a>(stream: &'a mut TcpStream) -> String {
    let mut buf = String::new();
    let mut b = [0; 1];
    while stream.read(&mut b).is_ok() {
        buf += str::from_utf8(&b).unwrap();
        if buf.ends_with("\r\n\r\n") {
            break
        }
    }
    println!("handle this: {:?}", buf);
    let mut parts = buf.split(' ');
    let _method = parts.next(); // http method: GET / POST / PUT
    let url = parts.next().unwrap(); // PATH
    return url.to_string()
}

fn handle_client(mut stream: TcpStream) {
    let url = read_stream(&mut stream);
    let filename = create_filename(url);
    let file_type = {
        if filename.ends_with("txt")  {
            "text/text".to_string()
        }
        else if filename.ends_with("html") || filename.ends_with("htm")  {
            "text/html".to_string()
        }
        else {
            "application/".to_string() + filename.split('.').last().unwrap_or("")
        }
    };
    let file_option = load_file(filename);

    let (http_code, data_len) = match file_option {
        Ok(ref data) => (format!("HTTP/1.1 200 OK"), data.metadata().unwrap().len() as usize),  // naked unwrap
        Err(ref _err) => (format!("HTTP/1.1 404 Not Found"), TEXT_404.len())
    };

    let headers = vec![
        http_code.as_ref(),
        "Server: ExperimentalRustyServer",
        "Connection: close",
        format!("Content-Type: {}", file_type).as_ref(),
        format!("Date: {} UTC", Utc::now().format("%a, %e %b %Y %T")).as_ref(),
        format!("Content-Length: {}", data_len).as_ref(),
    ].join("\n") + "\n\n";

    if let Err(err) = stream.write_all(headers.as_bytes()) {
        panic!("Error writing to buffer: {}", err)
    };

    match file_option {
        Ok(d) => {
            let mut by = io::BufReader::new(d);
            let mut buffer = [0; 10000];
            let mut bytes_read = 10000;
            while bytes_read == 10000 {
                bytes_read = by.read(&mut buffer).unwrap();
                stream.write_all(&buffer);
            }

            /*for place in buffer.iter_mut() {
                let byn = by.next();
                *place = byn.unwrap().unwrap();
            }*/
            /*loop {
                match by.next() {
                    None => break,
                    Some(ref v) => {
                        buffer[i] = v.unwrap();
                        i += 1;
                    },
                }
            }*/

            //let mut buffer2 = Vec::new();
            //let n = d.read_to_end(&mut buffer2).expect("Bad things");
            //by.read(&mut buffer);
            //let byc = &by.collect::<[u8, std::io::Error>>();
            //byc.unwrap():Weekday
        },  //naked unwrap
        Err(_e) => {
            stream.write_all(TEXT_404.as_bytes());
        }
    };
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}
