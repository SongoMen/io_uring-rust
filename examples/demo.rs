use rust_io_uring::fs::File;
use rust_io_uring::block_on;
use std::os::unix::io::AsRawFd;

fn main() {
    let file = std::fs::File::open("Cargo.toml").unwrap();
    let fd = file.as_raw_fd();
    let my_file = File { fd };
    let buf = vec![0u8; 1024];

    block_on(async {
        let (res, buf) = my_file.read_at(buf, 0).await;
        let n = res.unwrap();
        println!("Read: {:?}", String::from_utf8_lossy(&buf[..n]));
    });
}
