use rlimit::{setrlimit, Resource};
use std::mem;

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

const DEFAULT_SOFT_LIMIT: u64 = 4 * 1024 * 1024;
const DEFAULT_HARD_LIMIT: u64 = 8 * 1024 * 1024;
pub fn set_max_nofile() {
    setrlimit(Resource::FSIZE, DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT)
        .expect("Set `FSIZE` limit error");
    let limit = 1024 * 1024;
    setrlimit(Resource::NOFILE, limit, limit).expect("Set `NOFILE` limit error");
}
