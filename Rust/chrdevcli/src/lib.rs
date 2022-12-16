use std::io::{Write, BufWriter, Read};
use std::sync::mpsc;
use std::{fs::File, time::Duration};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, Shutdown};

// Take vector of undefined length greater than 2 and return array of arguments
pub fn get_arguments(args: Vec<String>) -> [String; 2] {
    if args.len() < 3 {
        panic!("Supplied not enough arguments. Found {}, expected 2\n", args.len());
    } else if args.len() > 3 {
        println!("Supplied {} arguments, expected 2. The rest will be discarded.\n", args.len());
    }
    let path = String::from(&args[1]);
    let keyword = String::from(&args[2]);
    return [path, keyword];
}

// Open File to read if valid
pub fn open_file_at_path(path: &String) -> File {
    let file = match File::open(&path) {
        Ok(file) => file,
        Err(e) => panic!("Unable to open file at {}. {:?}\n", &path, e),
    };
    return file;
}

// Set keyword
pub fn set_keyword(keyword: String) {
    let socket_address = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9001);
    loop {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &socket_address,
            Duration::new(5, 0)) {
            let mut buf = Vec::new();
            for byte in keyword.bytes() {
                buf.push(byte);
            }
            stream.write(&buf).expect("Things went to shit. They shouldn't.\n");
            stream.shutdown(Shutdown::Both).expect("Could not shut down stream");
            break;
        }
    }
}

// Get search results and write them to new file
pub fn write_results_from_stream_to_file(results: File) -> usize {
    let mut buffer = [0u8; 1024 * 512];
    let mut hits = 0usize;
    let mut received: usize = 0;
    let mut writer = BufWriter::new(results);
    let socket_address = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9002);
    'outer: loop {
        if let Ok(mut stream) = TcpStream::connect_timeout(
            &socket_address,
            Duration::new(9, 300)) {
            loop {
                if let Ok(n) = stream.read(&mut buffer[received..]) {
                    received += n;
                    if received % 8 == 0 {
                        for chunk in buffer[..received].chunks_exact(8) {
                            if chunk == [4u8; 8].as_slice() {
                                writer.flush().expect("Could not flush buffered writer. should never happen");
                                stream.shutdown(Shutdown::Both).expect("Could not shut down stream");
                                break 'outer 
                            };
                            if chunk.len() != 8 {println!("Encountered invalid byte in TcpStream"); continue}
                            let result_byte_str = format!("{}\n", usize::from_ne_bytes(chunk.try_into().expect("schmutz")));
                            hits += 1;
                            writer.write(result_byte_str.as_bytes()).expect("Failing... hard");
                        }
                        received = 0;
                    }

                }
            }
        }
    }
    return hits;
}

// Show progress how much has ben written to the character device
pub fn display_progress(rx: mpsc::Receiver<usize>, file_size: usize) -> bool {
    println!("Searching File ...\n");
    let mut progress_bar = String::from("[--------------------]");
    let mut bytes = 0;
    let mut i = 0;
    while  bytes < file_size {
        bytes = match rx.recv() {
            Ok(t) => t,
            Err(_) => continue,
        };
        let progress = (bytes as f32 / file_size as f32) * 100f32;
        if progress / 5 as f32 > i as f32 {
            i += 1;
            progress_bar.replace_range(i..i+1, "I");
        }
        print!("\r{} {:.2}%", progress_bar, progress);
    }
    return true
}
