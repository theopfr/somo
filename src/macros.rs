/// Pipe-safe stdout printing -- alternative to print!.
/// This macro exists because there is no way to handle pipes ending abruptly with print!
/// For more information, read:
/// - https://users.rust-lang.org/t/why-does-the-pipe-cause-the-panic-in-the-standard-library/107222/4
/// - rust-lang/rust#97889
///
/// # Usage
/// Same arguments & expected behaviour as print!, with graceful handling of (expected) IO errors.
#[macro_export]
macro_rules! sout {
    ($($arg:tt)*) => {{
        use std::io::Write;

        match write!(std::io::stdout(), $($arg)*) {
            Ok(_) => (),
            Err(broken_pipe) if broken_pipe.kind() == std::io::ErrorKind::BrokenPipe => (),
            Err(err) => panic!("Unknown error occurred while writing to stdout {:?}", err.kind()),
        }
    }};
}


/// Pipe-safe stdout printing -- alternative to println!.
/// This macro exists because there is no way to handle pipes ending abruptly with println!
/// For more information, read:
/// - https://users.rust-lang.org/t/why-does-the-pipe-cause-the-panic-in-the-standard-library/107222/4
/// - rust-lang/rust#97889
///
/// # Usage
/// Same arguments & expected behaviour as println!, with graceful handling of (expected) IO errors.
#[macro_export]
macro_rules! soutln {
    () => {{
        $crate::sout!("\n");
    }};
    ($($arg:tt)*) => {{
        $crate::sout!("{}\n", format!($($arg)*));
    }};
}
