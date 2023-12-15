use close_file::Closable;
use std::io::Write;

#[test]
fn close() {
    const FILE_PATH: &str = "temp";

    let mut f = std::fs::File::create(FILE_PATH).unwrap();
    f.write_all("Hello, world!".as_bytes()).unwrap();
    f.close().unwrap();
    std::fs::remove_file(FILE_PATH).unwrap();
}
