mod mf;
mod mf_config;
mod string_utils;

use crate::mf::MFparser;
use crate::mf_config::MFConfig;
use flexi_logger::{FileSpec, Logger};
use iso8601_duration::Duration;
use std::fs::File;
use std::io::Read;
use log::{info, error};
use tokio::{task, time};
use toml;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config_path = std::env::args()
        .nth(1)
        .expect("no configfile path pattern given");

    //parse config file
    let mut file = File::open(config_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    //init logger
    let _logger = Logger::try_with_str("info")?
        .log_to_file(FileSpec::default().directory("log"))
        .print_message() 
        .start()?;
    
    info!("logger is initialised");
    //init parser
    let config: MFConfig = toml::from_str(&contents)?;
    let parser = MFparser::new(config);

    let forever = task::spawn(async move {
        let mut interval = time::interval(
            Duration::parse(&parser.config.run_every)
                .expect("can't parse the duration")
                .to_std(),
        );

        loop {
            interval.tick().await;
            info!("start a parsing");
            match parser.run().await{
                Ok(()) => info!("finished the parsing cycle"),
                Err(e) => error!("get an error during the parsing: {e}")
            }
        }
    });

    forever.await?;
    Ok(())
}
