#[cfg(feature = "log")]
#[macro_use]
mod log {
    macro_rules! fmc_log {
        (trace, $($arg:expr),*) => {
            ()
        };
    }
}

#[cfg(not(feature = "log"))]
#[macro_use]
mod log {
    macro_rules! fmc_log {
        ($level:ident, $($arg:expr),*) => {
            $({
                ()
            }
            )*
        };
    }
}

macro_rules! fmc_trace {
    ($($arg:expr),*) => {
        ()
    };
}
