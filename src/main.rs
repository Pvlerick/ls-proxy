use std::{
    env,
    fs::OpenOptions,
    io::{stdin, Write},
};

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/lsp-proxy.log")
        .expect("create/open failed");
    write!(file, "{:?}\n", args).expect("write failed");
    file.flush().expect("flush failed");

    loop {
        let mut buff = String::new();
        let stdin = stdin();
        stdin.read_line(&mut buff).expect("read from stdin failed");
        write!(file, "{}", buff).expect("write failed");
        file.flush().expect("flush failed");
    }
}
