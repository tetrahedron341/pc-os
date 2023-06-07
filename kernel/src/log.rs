use core::sync::atomic::{AtomicBool, Ordering};

use crate::{println, serial_println};
use alloc::string::String;
use conquer_once::spin::OnceCell;
use crossbeam_queue::{ArrayQueue, PushError};
use log::{Level, LevelFilter, Log};

static LOGGER: OnceCell<Logger> = OnceCell::uninit();

/// Initialize the `log` crate backend.
///
/// Must be called **exactly once** after allocation is set up.
pub fn init(serial_max_level: LevelFilter, console_max_level: LevelFilter, capacity: usize) {
    let logger = LOGGER.get_or_init(|| Logger {
        serial_max_level,
        console_max_level,

        auto_flush: AtomicBool::new(true),

        log_queue: ArrayQueue::new(capacity),
    });

    log::set_logger(logger).expect("`crate::log::init()` called more than once");
    log::set_max_level(LevelFilter::Trace);
}

/// Sets whether or not to block and write log messages to console as soon as they are logged.
pub fn set_auto_flush(auto_flush: bool) {
    let logger = LOGGER.get().unwrap();
    logger.auto_flush.store(auto_flush, Ordering::Release);
}

/// Acts as a backend for the `log` crate. Sends logs to the VGA console and/or to the serial interface.
///
/// ## Race condition
/// If either the VGA console or serial buffer may be locked by the current thread
/// during a log, perform a flush before the lock is obtained.
///
/// Performing a flush **while** either interface is locked by the current thread will trigger a deadlock.
struct Logger {
    serial_max_level: LevelFilter,
    console_max_level: LevelFilter,

    auto_flush: AtomicBool,

    log_queue: ArrayQueue<(String, Level)>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.serial_max_level || metadata.level() <= self.console_max_level
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let message = alloc::format!(
                "[{}] {} - {}",
                record.level(),
                record.metadata().target(),
                record.args()
            );
            if let Err(PushError(record)) = self.log_queue.push((message, record.level())) {
                self.flush();
                self.log_queue.push(record).unwrap();
            }
        }

        if self.auto_flush.load(Ordering::Acquire) {
            self.flush();
        }
    }
    fn flush(&self) {
        while let Ok((record, level)) = self.log_queue.pop() {
            if level <= self.serial_max_level {
                serial_println!("{}", record);
            }
            if level <= self.console_max_level {
                println!("{}", record);
            }
        }
    }
}

pub fn flush() {
    ::log::logger().flush();
}

pub async fn flush_routine() {
    loop {
        flush();
        crate::task::timer::wait_n_ticks(1).await;
    }
}
