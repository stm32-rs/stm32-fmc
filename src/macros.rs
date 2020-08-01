#[cfg(feature = "log")]
#[macro_use]
mod log {
    macro_rules! fmc_log {
        (trace, $($arg:expr),*) => { log::trace!($($arg),*); };
    }
}

#[cfg(not(feature = "log"))]
#[macro_use]
mod log {
    macro_rules! fmc_log {
        ($level:ident, $($arg:expr),*) => { $( let _ = $arg; )* }
    }
}

macro_rules! fmc_trace {
    ($($arg:expr),*) => (fmc_log!(trace, $($arg),*));
}
