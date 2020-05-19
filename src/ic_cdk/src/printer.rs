use std::fmt;
use std::fmt::Write;
use std::io;
use std::panic;

use crate::api;

unsafe fn log(s: &str) {
    api::print(s)
}

fn _print(buf: &str) -> io::Result<()> {
    unsafe {
        log(buf);
    }
    Ok(())
}

/// This is a bit useless at the moment, but hopefully the runtime will
/// allow for eprint in the future and make it less useless
fn _eprint(buf: &str) -> io::Result<()> {
    _print(buf)
}

/// Used by the `print` macro
#[doc(hidden)]
pub fn _print_args(args: fmt::Arguments<'_>) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    let _ = _print(&buf);
}

/// Used by the `eprint` macro
#[doc(hidden)]
pub fn _eprint_args(args: fmt::Arguments<'_>) {
    let mut buf = String::new();
    let _ = buf.write_fmt(args);
    let _ = _eprint(&buf);
}

/// Emulates the default `print!` macro.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print_args(format_args!($($arg)*)));
}

/// Emulates the default `eprint!` macro.
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ($crate::_eprint_args(format_args!($($arg)*)));
}

/// Emulates the default `println!` macro.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::_print_args($crate::format_args!($($arg)*));
    })
}

/// Emulates the default `eprintln!` macro.
#[macro_export]
macro_rules! eprintln {
    () => ($crate::eprint!("\n"));
    ($($arg:tt)*) => ({
        $crate::_eprint_args($crate::format_args!($($arg)*));
    })
}

type PrintFn = fn(&str) -> io::Result<()>;

struct Printer {
    printfn: PrintFn,
    buffer: String,
    is_buffered: bool,
}

impl Printer {
    fn new(printfn: PrintFn, is_buffered: bool) -> Printer {
        Printer {
            buffer: String::new(),
            printfn,
            is_buffered,
        }
    }
}

impl io::Write for Printer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.push_str(&String::from_utf8_lossy(buf));

        if !self.is_buffered {
            (self.printfn)(&self.buffer)?;
            self.buffer.clear();

            return Ok(buf.len());
        }

        if let Some(i) = self.buffer.rfind('\n') {
            let buffered = {
                let (first, last) = self.buffer.split_at(i);
                (self.printfn)(first)?;

                String::from(&last[1..])
            };

            self.buffer.clear();
            self.buffer.push_str(&buffered);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        (self.printfn)(&self.buffer)?;
        self.buffer.clear();

        Ok(())
    }
}

// nightly_compiler is enabled by build.rs when the compiler is a nightly
#[cfg(nightly_compiler)]
fn set_print(printer: Box<Printer>) {
    io::set_print(Some(Box::new(printer)));
}

#[allow(clippy::boxed_local)]
#[cfg(not(nightly_compiler))]
fn set_print(_printer: Box<Printer>) {
    api::print("WARNING: Print hooks do not work without using an experimental rust compiler, print statements will not be displayed");
}

#[cfg(nightly_compiler)]
fn set_panic(printer: Box<Printer>) {
    io::set_panic(Some(Box::new(printer)));
}

#[allow(clippy::boxed_local)]
#[cfg(not(nightly_compiler))]
fn set_panic(_printer: Box<Printer>) {
    api::print("WARNING: Panic hooks do not work without using an experimental rust compiler, panic messages will not be displayed")
}

/// Sets a line-buffered stdout, uses debug.trace
pub fn set_stdout() {
    let printer = Printer::new(_print, true);
    set_print(Box::new(printer));
}

/// Sets a line-buffered stderr, uses debug.trace
pub fn set_stderr() {
    let eprinter = Printer::new(_eprint, true);
    set_panic(Box::new(eprinter));
}

/// Sets a custom panic hook, uses debug.trace
pub fn set_panic_hook() {
    panic::set_hook(Box::new(|info| {
        let file = info.location().unwrap().file();
        let line = info.location().unwrap().line();
        let col = info.location().unwrap().column();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let err_info = format!("Panicked at '{}', {}:{}:{}", msg, file, line, col);

        unsafe {
            log(&err_info);
        }
    }));
}

/// Sets stdout, stderr, and a custom panic hook
pub fn hook() {
    set_stdout();
    set_stderr();
    set_panic_hook();
}
