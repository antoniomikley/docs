use std::usize;
use std::{fs::File, env};
use std::io::{Write, Read};
use std::thread;
use std::sync::mpsc;
use chrdevcli::{display_progress, set_keyword, get_arguments, write_results_from_stream_to_file, open_file_at_path};

fn main() {
    // take arguments from command line
    const BUF_SIZE: usize = 1024 * 1024;
    let arguments = get_arguments(env::args().collect());
    let keyword = arguments[1].clone();
    let path = arguments[0].clone();
    // open the file to be searched and get its size
    let mut file = open_file_at_path(&path);
    let file_size: usize = file.metadata().unwrap().len().try_into().unwrap();
    let mut bytes_read = 0;
    // create file to which results will be written to
    let result_file = File::create("results")
        .expect("Could not create file to store results");
    let set_keyword = thread::spawn(|| set_keyword(keyword));
    let result_hits = thread::spawn(|| write_results_from_stream_to_file(result_file));
    // open character device in write mode
    let mut chrdev = File::create("/dev/test0")
        .expect("Should have been able to open /dev/test0");
    set_keyword.join().unwrap();
    // channel to send how many bytes have been written to the progress bar
    let (tx, rx) = mpsc::channel();
    // fancy progress bar
    let progress_bar = thread::spawn( move || display_progress(rx, file_size));
    let mut buffer = [0u8; BUF_SIZE];
    // write contents from file to character device 
    while bytes_read < file_size {
        let br = file.read(&mut buffer[0..]).unwrap();
        chrdev.write(&buffer[..br]).unwrap();
        bytes_read += br;
        _ = tx.send(bytes_read);
    }
    // does not need to be done, but it is anyway
    chrdev.flush().unwrap();
    if bytes_read > file_size {println!("Something is as it should not be")}
    // in the case that the file is exactly the same or a multiple of the buffer size, then the
    // character device needs to be told that there wont be written any more bytes
    if file_size % BUF_SIZE == 0 {
        chrdev.write(&[4u8; 1]).unwrap();
    }
    // wait for threads to finish
    progress_bar.join().unwrap();
    let results = result_hits.join().unwrap();
    println!("\nFound {} {} times.", arguments[1], results);
}

