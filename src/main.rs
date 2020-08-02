#![recursion_limit = "256"]

use log::info;
use tower_lsp::{LspService, Server};

mod completion;
mod server;
mod sources;
use server::Backend;

#[tokio::main]
async fn main() {
    flexi_logger::Logger::with_str("info").start().unwrap();
    info!("starting LSP server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(Backend::new());
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}
