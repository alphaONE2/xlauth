use clap::{Parser, Subcommand};
use keyring::Entry;
use std::{
    io::Write,
    net::{Ipv4Addr, SocketAddr, TcpStream},
    process::Command,
    time::{Duration, Instant},
};
use totp_rs::{Rfc6238, Secret, TOTP};
use zeroize::Zeroize;

const DEFAULT_NAME: &str = "<default>";
const DEFAULT_TIMEOUT: &str = "60s";
const DEFAULT_PATH: &str = "%LocalAppData%\\XIVLauncher\\XIVLauncher.exe";

fn main() {
    let mut cli: Cli = Cli::parse();
    match &mut cli.command {
        Commands::Save { name, secret } => save(name, secret),
        Commands::Delete { name } => delete(name),
        Commands::Launch {
            name,
            timeout,
            path,
        } => launch(name, timeout, path),
    }
}

fn save(name: &str, secret: &mut Vec<String>) {
    let validated_secret = validate_secret(secret);
    let mut encoded_secret = validated_secret.to_encoded().to_string();
    Entry::new("xlauth", name)
        .unwrap()
        .set_password(encoded_secret.as_str())
        .expect("TOTP secret was saved");
    encoded_secret.zeroize();
}

fn validate_secret(secret: &mut Vec<String>) -> Secret {
    println!("{:?}", secret);
    let length = secret.iter().fold(0, |l, s| l + s.len());
    let mut joined = String::new();
    joined.reserve(length);
    joined = secret.iter().fold(joined, |mut r, s| {
        r.push_str(s);
        r
    });
    joined.retain(|c| !c.is_whitespace());
    let decoded_secret = Secret::Encoded(joined)
        .to_bytes()
        .expect("TOTP secret is valid");
    secret.zeroize();
    Secret::Raw(decoded_secret)
}

fn load(name: &str) -> Vec<u8> {
    let entry = Entry::new("xlauth", name).unwrap();
    let encoded_secret = entry.get_password().expect("TOTP secret was loaded");
    Secret::Encoded(encoded_secret)
        .to_bytes()
        .expect("TOTP secret is valid")
}

fn delete(name: &str) {
    let entry = Entry::new("xlauth", name).unwrap();
    entry.delete_password().expect("TOTP secret was deleted");
}

fn launch(name: &str, timeout: &Duration, path: &str) {
    let rfc = Rfc6238::with_defaults(load(name)).unwrap();
    let totp = TOTP::from_rfc6238(rfc).unwrap();
    Command::new(expand_str::expand_string_with_env(path).unwrap())
        .spawn()
        .expect("XIV Launcher was started");
    send_totp(totp, timeout);
}

fn send_totp(totp: TOTP, timeout: &Duration) {
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 4646));
    let start = Instant::now();
    loop {
        match timeout.saturating_sub(Instant::now().saturating_duration_since(start)) {
            Duration::ZERO => break,
            remaining => match TcpStream::connect_timeout(&addr, remaining) {
                Err(_) => continue,
                Ok(mut stream) => {
                    let totp_code = totp.generate_current().unwrap();
                    let pkg_name = env!("CARGO_PKG_NAME");
                    let pkg_version = env!("CARGO_PKG_VERSION");
                    let request = format!(
                        "GET /ffxivlauncher/{totp_code} HTTP/1.0\r\n\
                        Host: localhost\r\n\
                        User-Agent: {pkg_name}/{pkg_version}\r\n\
                        Content-Length: 0\r\n\
                        \r\n"
                    );
                    stream.write_all(request.as_bytes()).unwrap();
                }
            },
        }
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save a TOTP secret
    Save {
        /// Name of the TOTP secret to save
        #[arg(short, long, default_value = DEFAULT_NAME)]
        name: String,

        /// TOTP secret
        #[arg(num_args = 1.., value_delimiter = ' ')]
        secret: Vec<String>,
    },

    /// Delete a TOTP secret
    Delete {
        /// Name of the TOTP secret to delete
        #[arg(short, long, default_value = DEFAULT_NAME)]
        name: String,
    },

    /// Launch FFXIV
    Launch {
        /// Name for the TOTP secret to use
        #[arg(short, long, default_value = DEFAULT_NAME)]
        name: String,

        /// Timeout duration (e.g. 250ms)
        #[arg(short, long, value_parser = humantime::parse_duration, default_value = DEFAULT_TIMEOUT)]
        timeout: Duration,

        /// Path to XIV Launcher
        #[arg(short, long, default_value = DEFAULT_PATH)]
        path: String,
    },
}

/*
MIT License

Copyright © alphaONE2

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/
