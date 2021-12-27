use std::borrow::Borrow;

const HEADERS: [&str; 13] = [
    "GET", "POST", "HEAD", "PUT", "COPY", "DELETE", "MOVE", "OPTIONS", "LINK", "UNLINK", "TRACE",
    "PATCH", "WRAPPED",
];

pub fn is_http_header(data: &[u8]) -> bool {
    for header in HEADERS {
        if data.starts_with(header.as_bytes()) {
            return true;
        }
    }
    return false;
}

pub fn bytes_contains(buff : &[u8], dest : &[u8]) -> bool {
    let s_len = dest.len();
    let mut count = 0;
    let mut i: usize = 0;
    for b in buff {
        if b == dest[i].borrow() {
            count += 1;
            if count >= s_len {
                return true;
            }
            i += 1;
        } else {
            count = 0;
            i = 0;
        }
    }
    return false;
}

#[test]
fn bytes_contains_test() {
    assert!(bytes_contains("Hello, world".as_bytes(), ", wor".as_bytes()));
    assert!(bytes_contains("Hello, world".as_bytes(), ", ".as_bytes()));
    assert!(bytes_contains("Hello, world".as_bytes(), "Hello".as_bytes()));
    assert!(!bytes_contains("Hello, world".as_bytes(), "la".as_bytes()));
}
