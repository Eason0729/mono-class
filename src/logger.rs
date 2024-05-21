use std::fs::OpenOptions;

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
    if verbose {
        exporter.push(TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ));
    }
    CombinedLogger::init(exporter).unwrap();
}
