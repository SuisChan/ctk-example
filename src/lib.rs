use std::time::{Duration, Instant};

use byteorder::{BigEndian, ByteOrder};
use reqwest::{
    header::{ACCEPT, CONTENT_TYPE, USER_AGENT},
    Method,
};
use sha1::{Digest, Sha1};

mod client_token;

pub use client_token::acquire_token;

const UA: &str = "Spotify/8.6.26 Android/29 (Pixel 4a, API 29)";
const X_PROTOBUF: &str = "application/x-protobuf";

pub(crate) fn solve_hash_cash(
    ctx: &[u8],
    prefix: &[u8],
    length: i32,
    dst: &mut [u8],
) -> std::io::Result<Duration> {
    let md = Sha1::digest(ctx);

    let now = Instant::now();

    let mut counter: i64 = 0;
    let target: i64 = BigEndian::read_i64(&md[12..20]);

    let suffix = loop {
        let suffix = [(target + counter).to_be_bytes(), counter.to_be_bytes()].concat();

        let mut hasher = Sha1::new();
        hasher.update(prefix);
        hasher.update(&suffix);
        let md = hasher.finalize();

        if BigEndian::read_i64(&md[12..20]).trailing_zeros() >= (length as u32) {
            break suffix;
        }

        counter += 1;
    };

    dst.copy_from_slice(&suffix);

    Ok(now.elapsed())
}

fn request(method: Method, url: &str, body: Option<Vec<u8>>) -> Vec<u8> {
    let request = reqwest::blocking::Client::new()
        .request(method, url)
        .header(ACCEPT, X_PROTOBUF)
        .header(USER_AGENT, UA)
        .header(CONTENT_TYPE, X_PROTOBUF);

    let request = if let Some(body) = body {
        request.body(body)
    } else {
        request
    };

    request
        .send()
        .expect("failed to send request")
        .bytes()
        .expect("failed to read response")
        .to_vec()
}

pub mod hex {
    use std::fmt::Write;

    pub(crate) fn decode(hex: &str) -> Vec<u8> {
        let mut bytes = Vec::new();
        for chunk in hex.as_bytes().chunks(2) {
            let byte = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
            bytes.push(byte);
        }
        bytes
    }

    pub(crate) fn encode(input: impl AsRef<[u8]>) -> String {
        let mut hex = String::new();
        for byte in input.as_ref() {
            write!(&mut hex, "{:02x?}", byte).unwrap();
        }

        hex
    }
}
