use log::info;
use fern::Dispatch;

use crate::utils::{Error, ErrorKind};

type Result<'a, T> = std::result::Result<T, Error>;

pub fn setup_logger() -> Result<'static, ()> {
    match fern::log_file("oxide.log") {
        Ok(file) => {
            match Dispatch::new()
                .chain(file)
                .level(log::LevelFilter::Debug)
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{}] {}",
                        record.level(),
                        message,
                    ))
                })
                .apply() {
                Ok(_) => {
                    info!("Logger setup");

                    Ok(())
                },
                Err(_) => Err(Error::new(ErrorKind::LogInitError, "Failed to initiate logging".to_string())),
            }

        },
        Err(_) => Err(Error::new(ErrorKind::LogInitError, "Failed to open/create log file".to_string())),
    }
}
