mod filters;
mod protocol;
mod tools;

use std::io::{self, BufRead, Write};

fn main() {
    // --version flag — install scripts check this
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("rtkmcp {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<protocol::types::Request>(&line) {
            Ok(req) => protocol::handler::dispatch(req),
            Err(e) => protocol::handler::parse_error(&e.to_string()),
        };

        if let Ok(json) = serde_json::to_string(&response) {
            let _ = out.write_all(json.as_bytes());
            let _ = out.write_all(b"\n");
            let _ = out.flush();
        }
    }
}
