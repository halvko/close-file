use close_file::Closable;
use std::io::Write;

#[test]
fn close() {
    let mut f = std::fs::File::create("temp").unwrap();
    f.write_all("Hello, world!".as_bytes());
    f.close();
}