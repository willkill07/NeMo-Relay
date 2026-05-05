// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! NeMo Flow coding-agent gateway sidecar.

mod adapters;
mod config;
mod error;
mod gateway;
mod installer;
mod model;
mod server;
mod session;

use clap::Parser;

use crate::config::{Cli, Command};

#[tokio::main]
async fn main() -> Result<(), error::SidecarError> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Install(command)) => installer::install(command),
        Some(Command::HookForward(command)) => installer::hook_forward(command).await,
        None => server::serve(cli.server).await,
    }
}
