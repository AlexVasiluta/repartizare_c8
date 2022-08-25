use clap::{Parser, Subcommand};
#[macro_use]
extern crate rocket;

mod db;

mod county;
mod specializare;
mod student;

mod server;
mod year_gen;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generator { year: i32 },
    Server,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    match cli.command {
        Commands::Generator { year } => {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(year_gen::do_year(year))?;
        }
        Commands::Server => {
            server::run_server()?;
        }
    };
    Ok(())
}
