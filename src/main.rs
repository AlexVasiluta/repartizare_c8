use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Args {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generator {
        year: i32,
    },
    Server {
        #[clap(long, default_value_t = String::from("./"))]
        path: String,
        #[clap(short, long, default_value_t = 8095)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    match cli.command {
        Commands::Generator { year } => {
            println!("Generating year {year}");
            repartizare_c8::year_gen::do_year(year).await?;
        }
        Commands::Server { path, port } => {
            println!("Starting server, listening on port {port}, serving from '{path}' ");
            repartizare_c8::server::run_server(path, port).await?;
        }
    };
    Ok(())
}
