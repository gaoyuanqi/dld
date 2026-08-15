use dld::cli;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::parse_args().await {
        eprintln!("\n失败信息：\n {e:#}");
        std::process::exit(1);
    }
}
