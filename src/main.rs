use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "lintropy", version, about = "Default CLI app")]
struct Cli {
    /// Name to print in the greeting.
    #[arg(default_value = "world")]
    name: String,
}

fn main() {
    let cli = Cli::parse();
    println!("Hello, {}!", cli.name);
}
