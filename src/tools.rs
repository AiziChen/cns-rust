use once_cell::sync::Lazy;
use regex::bytes::Regex;
use std::{borrow::Borrow, mem};

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
    let s_len = dest.len();
    let mut i: usize = 0;
    for b in buf {
        if b == dest[i].borrow() {
            i += 1;
            if i >= s_len {
                return true;
            }
        } else {
            i = 0;
        }
    }
    return false;
}

/**
 * Support `TCP_FASTOPEN` on Linux 3.7 and above
 */
pub fn enable_tcp_fastopen(sockfd: i32) -> bool {
    let queue: libc::c_int = 1;
    unsafe {
        let ret = libc::setsockopt(
            sockfd,
            libc::IPPROTO_TCP,
            libc::TCP_FASTOPEN,
            &queue as *const _ as *const libc::c_void,
            mem::size_of_val(&queue) as libc::socklen_t,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            eprintln!("Set `TCP_FASTOPEN` error: {:?}", err);
            return false;
        } else {
            return true;
        }
    }
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
