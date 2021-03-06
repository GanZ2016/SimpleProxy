#![feature(lookup_host)]
#![feature(mpsc_select)]

#[macro_use]
extern crate log;
extern crate time;

pub mod logger;
pub mod socks5;
pub mod server;
pub mod client;
pub mod timer;
mod protocol {
    pub const HEARTBEAT_INTERVAL_MS: u32 = 5000;
    pub const ALIVE_TIMEOUT_TIME_MS: i64 = 60000;

    pub mod cs {
        pub const OPEN_PORT: u8 = 1;
        pub const CLOSE_PORT: u8 = 2;
        pub const SHUTDOWN_WRITE: u8 = 4;
        pub const CONNECT: u8 = 5;
        pub const CONNECT_DOMAIN_NAME: u8 = 6;
        pub const DATA: u8 = 7;
        pub const HEARTBEAT: u8 = 8;
    }

    pub mod sc {
        pub const CLOSE_PORT: u8 = 1;
        pub const SHUTDOWN_WRITE: u8 = 3;
        pub const CONNECT_OK: u8 = 4;
        pub const DATA: u8 = 5;
        pub const HEARTBEAT_RSP: u8 = 6;
    }
}