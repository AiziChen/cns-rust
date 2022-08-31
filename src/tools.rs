use once_cell::sync::Lazy;
use regex::bytes::Regex;

const METHOD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^GET|^POST|^HEAD|^PUT|^COPY|^DELETE|^MOVE|^OPTIONS|^LINK|^UNLINK|^TRACE|^PATCH|^WRAPPED",
    )
    .unwrap()
});

pub fn is_http_header(data: &[u8]) -> bool {
    return METHOD_RE.is_match(data);
}

pub fn bytes_contains(buf: &[u8], dest: &[u8]) -> bool {
    let dlen = dest.len();
    let mut di: usize = 0;
    for b in buf {
        if b == &dest[di] {
            di += 1;
            if di >= dlen {
                return true;
            }
        } else {
            di = 0;
        }
    }
    return false;
}

#[allow(unused)]
pub fn bytes_contains2(buf: &[u8], dest: &[u8]) -> bool {
    return match find_subsequence(buf, dest) {
        Some(_) => true,
        None => false,
    };
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    return haystack
        .windows(needle.len())
        .position(|window| window == needle);
}

#[test]
fn bytes_contains_test() {
    assert!(bytes_contains(
        "Hello, world".as_bytes(),
        ", wor".as_bytes()
    ));
    assert!(bytes_contains("Hello, world".as_bytes(), ", ".as_bytes()));
    assert!(bytes_contains(
        "Hello, world".as_bytes(),
        "Hello".as_bytes()
    ));
    assert!(!bytes_contains("Hello, world".as_bytes(), "la".as_bytes()));
}
