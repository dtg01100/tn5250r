// Cross-platform TCP keepalive socket option logic for TN5250R

#[cfg(unix)]
pub fn enable_tcp_keepalive(socket: std::os::unix::io::RawFd) -> Result<(), std::io::Error> {
    let optval: libc::c_int = 1;
    let ret = unsafe {
        libc::setsockopt(
            socket,
            libc::SOL_SOCKET,
            libc::SO_KEEPALIVE,
            &optval as *const _ as *const libc::c_void,
            std::mem::size_of_val(&optval) as libc::socklen_t,
        )
    };
    if ret != 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
pub fn enable_tcp_keepalive(socket: u64) -> Result<(), std::io::Error> {
    use winapi::um::winsock2::{setsockopt, SOCKET, SOL_SOCKET, SO_KEEPALIVE, SOCKET_ERROR};
    let optval: i32 = 1;
    let ret = unsafe {
        setsockopt(
            socket as SOCKET,
            SOL_SOCKET,
            SO_KEEPALIVE,
            &optval as *const _ as *const i8,
            std::mem::size_of_val(&optval) as i32,
        )
    };
    if ret == SOCKET_ERROR {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}
