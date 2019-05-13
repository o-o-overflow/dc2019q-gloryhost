#[macro_use]
extern crate slog;
#[macro_use]
extern crate wasmer_runtime as wasm_r;
extern crate wasmer_wasi as wasm_i;

use std::fs::File;
use std::mem;
use std::ptr;
use std::time::Duration;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use base64;
use futures::{Future, Stream};
use libc::c_int;
use slog::{Drain, Logger};
use slog_term::{FullFormat, PlainSyncDecorator};
use syscallz::{Action, Context, Syscall};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use wasm_r::Func;

extern "C" {
    static mut _data_size: i32;
    static mut _data2: [u8; 64];
    static mut _data3: [u8; 16];
    static mut _data4: [u8; 64];
    static mut _data5: [u8; 256 * 512];
    static mut _data6: [u8; 64];
    fn init_data();
    fn check_data(offset: c_int);
}

const WELCOME_MESSAGE: &str = include_str!("welcome.txt");
const TIMEOUT_SECS: u64 = 1;

/// Host information.
#[derive(Debug, Clone)]
struct HostInfo {
    major_version: usize,
    minor_version: usize,
    git_commit: String,
}

impl HostInfo {
    fn new(major_version: usize, minor_version: usize, git_commit: String) -> HostInfo {
        HostInfo {
            major_version,
            minor_version,
            git_commit,
        }
    }
}

/// Main.
fn main() {
    // TODO: Issue and check challenge

    let log = new_logger();
    let host_info = Box::new(HostInfo::new(
        env!("CARGO_PKG_VERSION_MAJOR").parse().expect("parse"),
        env!("CARGO_PKG_VERSION_MINOR").parse().expect("parse"),
        "25149a8c3b0fd8c6f09b3d43b8f751800d16828d".to_owned(),
    ));

    unsafe {
        let mut flag = String::new();
        File::open("flag")
            .expect("unable to open flag")
            .read_to_string(&mut flag)
            .expect("unable to read flag");
//        debug!(
//            log,
//            "secret={:#?} array1={:#?}",
//            _data6.as_ptr(),
//            _data3.as_ptr(),
//        );
        ptr::copy_nonoverlapping(flag.as_ptr(), _data6.as_mut_ptr(), flag.len());
        init_data();
    }

    let server_addr = "0.0.0.0:9999"
        .parse()
        .expect("unable to parse server address");
    let listener = TcpListener::bind(&server_addr).expect("unable to bind listener");
    info!(log, "gloryhosting on {}", server_addr);
    let server = listener
        .incoming()
        .for_each(move |socket| {
            let client = on_client(&*host_info as *const _ as usize, socket);
            tokio::spawn(client);
            Ok(())
        })
        .map_err(move |e| {
            error!(log, "unable to accept remote peers: {}", e);
        });

    filter_syscalls().expect("unable to load seccomp filter");
    tokio::run(server);
}

/// Return a logger.
fn new_logger() -> Logger {
    Logger::root(
        FullFormat::new(PlainSyncDecorator::new(std::io::stdout()))
            .build()
            .fuse(),
        o!(),
    )
}

/// Initialize the syscall filter.
fn filter_syscalls() -> syscallz::Result<()> {
    let syscalls = [
        Syscall::accept,
        Syscall::accept4,
        Syscall::brk,
        Syscall::clone,
        Syscall::close,
        Syscall::epoll_create1,
        Syscall::epoll_ctl,
        Syscall::epoll_wait,
        Syscall::futex,
        Syscall::getpeername,
        Syscall::getrandom,
        Syscall::ioctl,
        Syscall::madvise,
        Syscall::mmap,
        Syscall::mprotect,
        Syscall::mremap,
        Syscall::munmap,
        Syscall::open,
        Syscall::openat,
        Syscall::pipe2,
        Syscall::prctl,
        Syscall::read,
        Syscall::recvfrom,
        Syscall::rt_sigaction,
        Syscall::rt_sigprocmask,
        Syscall::sched_getaffinity,
        Syscall::sched_yield,
        Syscall::sendto,
        Syscall::set_robust_list,
        Syscall::sigaltstack,
        Syscall::write,
    ];

    let mut c = Context::init_with_action(Action::Kill)?;
    for syscall in &syscalls {
        c.allow_syscall(*syscall)?;
    }

    c.load()
}

/// Handle a client.
fn on_client(host_info: usize, socket: TcpStream) -> impl Future<Item = (), Error = ()> {
    let peer_addr = socket
        .peer_addr()
        .map(|x| x.to_string())
        .unwrap_or_else(|_| "UNKNOWN".to_owned());
    let log = new_logger().new(o!("peer" => peer_addr));
    info!(log, "servicing client");

    let client = io::write_all(
        socket,
        format!(
            r#"
                               WELCOME TO THE


{}

      (where you can freely run code for the glory of humankind, obviously)

              COMPUTE WHAT THOU WILT SHALL BE THE WHOLE OF THE TOS

                     (well, that and a {} second timeout)

              SHOW ME WHAT YOU'VE GOT I WANT TO SEE WHAT YOU'VE GOT

                     (show me a base64-encoded WASI binary)

"#,
            WELCOME_MESSAGE, TIMEOUT_SECS
        ),
    )
    .and_then(|(socket, _)| io::read_to_end(socket, vec![]))
    .and_then({
        let log = log.clone();
        move |(socket, encoded_data)| {
            let data = match base64::decode(&encoded_data) {
                Ok(x) => x,
                Err(e) => {
                    let message = format!("your code is wack: {}", e);
                    error!(log, "{}", message);
                    return io::write_all(socket, format!("{}\n", message));
                }
            };

            info!(log, "executing module");
            match execute_module(host_info, &data) {
                Ok(x) => {
                    info!(log, "module returned {:x}", x);
                    io::write_all(socket, format!("YOU'VE GOT {:x}\n", x))
                }

                Err(e) => {
                    let message = format!("your code makes me feel funny: {}", e);
                    error!(log, "{}", message);
                    io::write_all(socket, format!("{}\n", message))
                }
            }
        }
    })
    .and_then(|_| Ok(()))
    .timeout(Duration::from_secs(TIMEOUT_SECS))
    .map_err({
        let log = log.clone();
        move |_| {
            error!(log, "unable to service remote peer");
        }
    });

    return client;
}

/// Execute a module.
fn execute_module(host_info: usize, data: &[u8]) -> Result<i32, String> {
    let mut base_imports = wasm_i::generate_import_object(vec![], vec![], vec![]);
    let custom_imports = imports! {
        "env" => {
            "get_host_info" => func!(wasi_get_host_info),
            "get_data_size" => func!(wasi_get_data_size),
            "get_data2" => func!(wasi_get_data2),
            "get_data3" => func!(wasi_get_data3),
            "get_data4" => func!(wasi_get_data4),
            "get_data5" => func!(wasi_get_data5),
            "debug_ts" => func!(wasi_debug_ts),
            "debug_flush" => func!(wasi_debug_flush),
            "check_data" => func!(wasi_check_data),
            "debug_read" => func!(wasi_debug_read),
        },
    };

    base_imports.extend(custom_imports);
    let mut module = wasm_r::instantiate(data, &base_imports)
        .map_err(|e| format!("unable to instantiate module: {}", e))?;
    module.context_mut().data = unsafe { mem::transmute(host_info) };

    let main: Func<(), i32> = module
        .func("this_is_what_ive_got")
        .map_err(|e| format!("unable to resolve entry point: {}", e))?;
    let value = main
        .call()
        .map_err(|e| format!("unable to execute entry point: {}", e))?;
    Ok(value)
}

/// Return host info pointer.
fn wasi_get_host_info(ctx: &mut wasm_r::Ctx) -> i64 {
    ctx.data as i64
}

/// Return pointer to array1_size.
fn wasi_get_data_size(_: &mut wasm_r::Ctx) -> i64 {
    unsafe { &_data_size as *const _ as i64 }
}

/// Return garbage.
fn wasi_get_data2(_: &mut wasm_r::Ctx) -> i64 {
    unsafe { _data2.as_ptr() as i64 }
}

/// Return pointer to array1.
fn wasi_get_data3(_: &mut wasm_r::Ctx) -> i64 {
    unsafe { _data3.as_ptr() as i64 }
}

/// Return garbage.
fn wasi_get_data4(_: &mut wasm_r::Ctx) -> i64 {
    unsafe { _data4.as_ptr() as i64 }
}

/// Return pointer to array2.
fn wasi_get_data5(_: &mut wasm_r::Ctx) -> i64 {
    unsafe { _data5.as_ptr() as i64 }
}

#[cfg(target_arch = "x86_64")]
/// rdtscp
fn wasi_debug_ts(_: &mut wasm_r::Ctx) -> u64 {
    unsafe {
        let mut tmp = 0;
        __rdtscp(&mut tmp)
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn wasi_debug_ts(_: &mut wasm_r::Ctx) -> u64 {
    unimplemented!()
}

#[cfg(target_arch = "x86_64")]
/// clflush
fn wasi_debug_flush(_: &mut wasm_r::Ctx, addr: i64) {
    unsafe {
        _mm_clflush(addr as *mut u8);
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn wasi_debug_flush(_: &mut wasm_r::Ctx, _: i64) {
    unimplemented!()
}

/// Victim function.
fn wasi_check_data(_: &mut wasm_r::Ctx, offset: i32) {
    unsafe {
        check_data(offset);
    }
}

/// Probe.
fn wasi_debug_read(_: &mut wasm_r::Ctx, addr: i64) {
    unsafe {
        let _ = (addr as *const u8).read_volatile();
    }
}
