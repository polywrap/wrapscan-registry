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
