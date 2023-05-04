use log::{Log, Level, LevelFilter};
use crate::{println, serial_println};
use crossbeam_queue::{ArrayQueue, PushError};
use alloc::string::String;
use alloc::boxed::Box;

/// Initialize the `log` crate backend.
/// 
/// Must be called **exactly once** after allocation is set up.
pub fn init(serial_min_level: LevelFilter, console_min_level: LevelFilter, capacity: usize) {
    let logger = Box::new(Logger {
        serial_min_level,
        console_min_level,

        log_queue: ArrayQueue::new(capacity)
    });

    log::set_logger(Box::leak(logger)).expect("`crate::log::init()` called more than once");
    log::set_max_level(LevelFilter::Info);
}

/// Acts as a backend for the `log` crate. Sends logs to the VGA console and/or to the serial interface.
/// 
/// ## Race condition
/// If either the VGA console or serial buffer may be locked by the current thread 
/// during a log, perform a flush before the lock is obtained. 
/// 
/// Performing a flush **while** either interface is locked by the current thread will trigger a deadlock.
struct Logger {
    serial_min_level: LevelFilter,
    console_min_level: LevelFilter,

    log_queue: ArrayQueue<(String, Level)>,
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.serial_min_level ||
        metadata.level() <= self.console_min_level
    }
    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let message = alloc::format!("[{}] {} - {}", record.level(), record.metadata().target(), record.args());
            if let Err(PushError(record)) = self.log_queue.push((message, record.level())) {
                self.flush();
                self.log_queue.push(record).unwrap();
            }
        }
    }
    fn flush(&self) {
        while let Ok((record, level)) = self.log_queue.pop() {
            if level <= self.serial_min_level {
                serial_println!("{}", record);
            }
            if level <= self.console_min_level {
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