use log::info;
use fern::Dispatch;

use anyhow;

pub fn setup_logger() -> anyhow::Result<()> {
    Dispatch::new()
        .chain(fern::log_file("oxide.log")?)
        .level(log::LevelFilter::Debug)
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {}",
                record.level(),
                message,
            ))
        })
        .apply()?;

    info!("Logger setup");

    Ok(())
}
