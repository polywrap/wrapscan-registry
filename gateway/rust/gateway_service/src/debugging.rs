#[cfg(feature = "local")]
#[macro_export]
macro_rules! debug {
    ($($x:tt)*) => { dbg!($($x)*) }
}

#[cfg(not(feature = "local"))]
#[macro_export]
macro_rules! debug {
    ($($x:tt)*) => {{}};
}

#[cfg(feature = "local")]
#[macro_export]
macro_rules! debug_println {
    ($($x:tt)*) => { println!($($x)*) }
}

#[cfg(not(feature = "local"))]
#[macro_export]
macro_rules! debug_println {
    ($($x:tt)*) => {{}};
}

#[cfg(feature = "local")]
pub fn log_error<E: std::fmt::Debug>(e: E) -> E {
    debug_println!("Error: {:?}", &e);
    e
}

#[cfg(not(feature = "local"))]
pub fn log_error<E: std::fmt::Debug>(e: E) -> E {
    debug!(&e);
    e
}
