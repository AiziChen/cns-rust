use log::warn;
use mimalloc::MiMalloc;
use rlimit::{setrlimit, Resource};
use tokio::net::TcpListener;

#[global_allocator]
static GLOABL: MiMalloc = MiMalloc;

/**
 * Support `TCP_FASTOPEN` on Linux 3.7 and above
 */
#[cfg(unix)]
pub fn enable_tcp_fastopen(stream: &TcpListener) -> bool {
    use std::{mem, os::unix::prelude::AsRawFd};

    let queue: libc::c_int = 1;
    unsafe {
        let ret = libc::setsockopt(
            stream.as_raw_fd(),
            libc::IPPROTO_TCP,
            libc::TCP_FASTOPEN,
            &queue as *const _ as *const libc::c_void,
            mem::size_of_val(&queue) as libc::socklen_t,
        );
        if ret < 0 {
            let err = std::io::Error::last_os_error();
            warn!("Set `TCP_FASTOPEN` error: {:?}", err);
            return false;
        } else {
            return true;
        }
    }
}

const DEFAULT_SOFT_LIMIT: u64 = 4 * 1024 * 1024;
const DEFAULT_HARD_LIMIT: u64 = 8 * 1024 * 1024;
#[cfg(unix)]
pub fn set_max_nofile() {
    setrlimit(Resource::FSIZE, DEFAULT_SOFT_LIMIT, DEFAULT_HARD_LIMIT)
        .expect("Set `FSIZE` limit error");
    let limit = 1024 * 1024;
    setrlimit(Resource::NOFILE, limit, limit).expect("Set `NOFILE` limit error");
}
