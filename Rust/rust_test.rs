//! Test module in Rust

use kernel::{
    io_buffer::{IoBufferReader, IoBufferWriter},
    prelude::*,
    sync::{smutex::Mutex, Arc, ArcBorrow},
    {file, miscdev},
    net::{self, TcpListener, SocketAddr, SocketAddrV4, Ipv4Addr, TcpStream}
};
use alloc::str;

module! {
    type: Test,
    name: "test",
    license: "GPL",
    params: {
        nr_devs: u32 {
            default: 1,
            permissions: 0o644,
            description: "Number of test devices",
        },
    },
}

fn get_tcp_stream_on_port(port: u16) -> TcpStream {
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOOPBACK, port));
    let listener = TcpListener::try_new(net::init_ns(), &addr)
        .expect("Could not create TcpListener");
    loop {
        let _stream = match listener.accept(true) {
            Ok(stream) => {
                pr_info!("Got stream on port {}", port);
                return stream
            },
            Err(_) => {
                pr_info!("Could not establish stream on 127.0.0.1:{}... trying again...\n", port);
                continue;
            },
        }; 
    }
}

fn get_keyword_from_stream(stream: TcpStream) -> Vec<u8> {
    let mut keyword = Vec::new();
    let mut buffer = [0u8; 1024];
    let len = stream.read(&mut buffer, true)
        .expect("Could not retrieve keyword from stream");
    for i in 0..len {
        keyword.try_push(buffer[i]).unwrap();
    }
    pr_info!("got word from stream: {:?}",keyword);
    return keyword;
}

// struct for the actual device with the state it holds
struct Device {
    contents: Mutex<Vec<u8>>,
    stuff: Mutex<Vec<usize>>,
    keyword: Mutex<Vec<u8>>,
    result_stream: Mutex<Vec<TcpStream>>,
    results: Mutex<Vec<u8>>,
}

struct Test {
    _devs: Vec<Pin<Box<miscdev::Registration<Test>>>>,
}


#[vtable]
impl file::Operations for Test {
    type OpenData = Arc<Device>;
    type Data = Arc<Device>;

    fn open(context: &Arc<Device>, file: &file::File) -> Result<Arc<Device>> {
        pr_info!("File for device was opened\n");
        if file.flags() & file::flags::O_ACCMODE == file::flags::O_WRONLY {
            // make sure that all remaining stuff of the previous search is cleaned up
            context.contents.lock().clear();
            context.stuff.lock().clear();
            context.keyword.lock().clear();
            context.result_stream.lock().clear();
            context.results.lock().clear();
            // Set keyword
            let mut vec = context.keyword.lock();
            let word = get_keyword_from_stream(get_tcp_stream_on_port(9001));
            for i in 0..word.len() {
                vec.try_push(word[i])?;
            }
            // Initialize the buffer now to avoid new allocations for each write call
            let mut contents = context.contents.lock();
            contents.try_resize(1024*1024, 0u8)?;
            // Create counters that need to persist across write calls
            let mut counters = context.stuff.lock();
            counters.try_push(0)?;
            counters.try_push(vec.len())?;
            counters.try_push(0)?;
            counters.try_push(0)?;
            // get stream to send back search results
            let mut stream = context.result_stream.lock();
            stream.try_push(get_tcp_stream_on_port(9002))?;
        }
        Ok(context.clone())
    }

    fn read(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        _writer: &mut impl IoBufferWriter,
        _offset: u64,
        ) -> Result<usize> {
        // Display results of the last search in the kernel log if device file is read.
        let mut len: usize = 1;
        let keyword_hits = data.stuff.lock()[3];
        let bytes_read = data.stuff.lock()[0];
        pr_info!("{keyword_hits}, {bytes_read}\n");
        len -= 1;
        Ok(len)
    }

    fn write(
        data: ArcBorrow<'_, Device>,
        _file: &file::File,
        reader: &mut impl IoBufferReader,
        _offset: u64,
        ) -> Result<usize> { unsafe {
        let len = reader.len();
        let mut vec = data.contents.lock();
        reader.read_slice(&mut vec[..len])?;
        let mut counter = data.stuff.lock();
        let mut slice = *counter.get_unchecked(2);
        let kw_len = *counter.get_unchecked(1);
        let mut loc_byte = *counter.get_unchecked(0);
        let mut word_hits = *counter.get_unchecked(3);
        let mut results = data.results.lock();
        let keyword = data.keyword.lock();
        let stream = &data.result_stream.lock()[0];
        // actually search for the needle in the haystack
        // if i ever goes out of bounds... stuff happens
        for i in 0..len {
            loc_byte += 1;
            let ch = *vec.get_unchecked(i);
            if ch != *keyword.get_unchecked(slice) {
                slice = 0;
            }
            // need to check it twice, just to be sure
            if ch == *keyword.get_unchecked(slice) {
            slice += 1;
            }
            if slice != kw_len {
                continue
            }
            slice = 0;
            word_hits += 1;
            // accumulate results before sending them back
            if results.len() >= 1024 * 128 {
                stream.write(&results[..], true)?;
                results.clear();
            }
            results.try_extend_from_slice(&(loc_byte - kw_len).to_ne_bytes()[..])?;
            
        }
        // needs to be done so counters stay persistent
        counter[0] = loc_byte;
        counter[2] = slice;
        counter[3] = word_hits;
        // send back results if its the last write call
        if len < 1024 * 1024 {
            results.try_extend_from_slice([4u8; 8].as_slice())?;
            stream.write(&results[..], true)?;
        };
        Ok(len)
    }}
}

impl kernel::Module for Test {
    // init stuff. go figure
    fn init(_name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        let count = {
            let lock = module.kernel_param_lock();
            (*nr_devs.read(&lock)).try_into()?
        };
        pr_info!("Hello world, {} devices!\n", count);
        let mut devs = Vec::try_with_capacity(count)?;
        for i in 0..count {
            let dev = Arc::try_new(Device {
                stuff: Mutex::new(Vec::new()),
                contents: Mutex::new(Vec::try_with_capacity(1024*1024)?),
                keyword: Mutex::new(Vec::new()),
                result_stream: Mutex::new(Vec::new()),
                results: Mutex::new(Vec::new()),
            })?;
            let reg = miscdev::Registration::new_pinned(fmt!("test{i}"), dev)?;
            devs.try_push(reg)?;
        }
        Ok(Self { _devs: devs })
    }
}
