use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod bounds;
mod map;

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Computes and outputs ground footprint and 3D bounds of a GLTF model.
    Bounds(Bounds),
    /// Computes and outputs hash of a Digital Extinction map.
    MapHash(MapHash),
}

#[derive(Args)]
struct Bounds {
    #[clap(short, long, value_parser, help = "Path of a GLTF file.")]
    path: PathBuf,
}

#[derive(Args)]
struct MapHash {
    #[clap(
        short,
        long,
        value_parser,
        help = "Path of a Digital Extinction map file."
    )]
    path: PathBuf,
    #[clap(short, long, help = "Check validity of the file name.")]
    check: bool,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Bounds(args) => bounds::execute(args.path.as_path()),
        Command::MapHash(args) => map::execute(args.path.as_path(), args.check),
    }
}
