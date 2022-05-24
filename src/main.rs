use geng::prelude::*;
use std::collections::VecDeque;

mod client;
mod common;
#[cfg(not(target_arch = "wasm32"))]
mod server;

use client::Client;
use common::*;
#[cfg(not(target_arch = "wasm32"))]
use server::Server;

#[derive(clap::Parser, Clone)]
pub struct Opt {
    #[clap(long)]
    server: Option<String>,
    #[clap(long)]
    connect: Option<String>,
}

fn main() {
    // logger::init().unwrap();
    let mut opt: Opt = program_args::parse();
    if opt.connect.is_none() && opt.server.is_none() {
        if cfg!(target_arch = "wasm32") {
            opt.connect = Some(
                option_env!("CONNECT")
                    .expect("Set CONNECT compile time env var")
                    .to_owned(),
            );
        } else {
            opt.server = Some("127.0.0.1:1155".to_owned());
            opt.connect = Some("ws://127.0.0.1:1155".to_owned());
        }
    }
    if opt.server.is_some() && opt.connect.is_none() {
        #[cfg(not(target_arch = "wasm32"))]
        geng::net::Server::new(Server::new(), opt.server.as_deref().unwrap()).run();
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        let server = if let Some(addr) = &opt.server {
            let server = geng::net::Server::new(Server::new(), addr);
            let server_handle = server.handle();
            let server_thread = std::thread::spawn(move || {
                server.run();
            });
            Some((server_handle, server_thread))
        } else {
            None
        };

        let geng = Geng::new_with(geng::ContextOptions {
            title: "Yeti Paint".to_owned(),
            antialias: false,
            ..default()
        });
        let state = geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            {
                let connection = geng::net::client::connect(opt.connect.as_deref().unwrap());
                async move {
                    let mut connection = connection.await;
                    let message = connection.next().await;
                    match message {
                        Some(ServerMessage::Initial(state)) => (state, connection),
                        _ => unreachable!(),
                    }
                }
            },
            {
                let geng = geng.clone();
                move |(initial_state, connection)| Client::new(&geng, initial_state, connection)
            },
        );
        geng::run(&geng, state);

        #[cfg(not(target_arch = "wasm32"))]
        if let Some((server_handle, server_thread)) = server {
            server_handle.shutdown();
            server_thread.join().unwrap();
        }
    }
}
