use std::{fmt::Display, net::Ipv4Addr};

use anyhow::Result;
use clap::Args;

use roblox_studio_utils::RobloxStudioOpener;

pub type CliResult = Result<()>;

const DEFAULT_SERVER_ADDR: Ipv4Addr = Ipv4Addr::LOCALHOST;
const DEFAULT_SERVER_PORT: u16 = 50608;

#[derive(Debug, Clone, Copy)]
pub struct Context {
    quiet: bool,
    background: bool,
}

impl Context {
    pub const fn new(quiet: bool, background: bool) -> Self {
        Self { quiet, background }
    }

    pub fn opener(self) -> RobloxStudioOpener {
        let opener = RobloxStudioOpener::new();
        if self.background {
            opener.in_background()
        } else {
            opener
        }
    }

    pub fn print(self, message: impl Display) {
        if !self.quiet {
            println!("{message}");
        }
    }
}

#[derive(Args, Clone, Debug)]
pub struct ServerOptions {
    #[arg(long, default_value_t = DEFAULT_SERVER_ADDR)]
    pub server_addr: Ipv4Addr,
    #[arg(long, default_value_t = DEFAULT_SERVER_PORT)]
    pub server_port: u16,
}

impl ServerOptions {
    pub fn apply(&self, opener: RobloxStudioOpener) -> RobloxStudioOpener {
        opener
            .with_server_addr(self.server_addr)
            .with_server_port(self.server_port)
    }

    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.server_addr, self.server_port)
    }
}
