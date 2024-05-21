mod lang;
mod logger;
mod map;
mod resolve;

use std::path::PathBuf;

use clap::Parser;

/// A bundler for Java
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to source file containing desired class
    file: PathBuf,

    /// output location
    #[arg(short, long, default_value = "Output.java")]
    output: PathBuf,

    /// verbose mode
    #[arg(short, long)]
    verbose: bool,
}

fn main() {
    let args = Args::parse();

    logger::init_logger(args.verbose);

    log::info!("Starting bundler with args {:?}", args);
    println!(
        "Bundling all dependency in {} to {}",
        args.file.display(),
        args.output.display()
    );

    smol::block_on(async {
        let content = resolve::resolve(args.file).await;

        println!("Dependency solved");

        let size = content.len();

        smol::fs::write(&args.output, content)
            .await
            .expect("failed to write output");

        println!(
            "{} bytes has been written to {}",
            size,
            args.output.display()
        );
    });
}
