use std::{fs::OpenOptions, process::exit};

use log::LevelFilter;
use simplelog::*;

pub fn init_logger(verbose: bool) {
    let log_file_path = homedir::get_my_home()
        .unwrap()
        .unwrap()
        .join(".mono-class.bundler.log");
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("fail to open log file");

    let mut exporter: Vec<Box<dyn SharedLogger>> = vec![WriteLogger::new(
        LevelFilter::Info,
        Config::default(),
        log_file,
    )];
    exporter.push(TermLogger::new(
        match verbose {
            true => LevelFilter::Debug,
            false => LevelFilter::Error,
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    ));
    CombinedLogger::init(exporter).unwrap();

    std::panic::set_hook(Box::new(|panic| {
        log::error!("{}", panic.to_string());

        exit(1);
    }));
}
