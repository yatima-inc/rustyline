//!Readline Implementation in Rust
//!
//!This implementation is based on [Antirez's Linenoise](https://github.com/antirez/linenoise)
//!
//!# Example
//!
//!Usage
//!
//!```
//!let readline = rustyline::readline(">> ");
//!match readline {
//!     Ok(line) => println!("Line: {:?}",line),
//!     Err(_)   => println!("No input"),
//! }
//!```
extern crate libc;
extern crate nix;

use std::io;
use std::io::{Write, Read, Error, ErrorKind};
use nix::errno::Errno;
use nix::Error::Sys;
use nix::sys::termios;
use nix::sys::termios::{BRKINT, ICRNL, INPCK, ISTRIP, IXON, OPOST, CS8, ECHO, ICANON, IEXTEN, ISIG, VMIN, VTIME};

pub mod readline_error;

/// Maximum buffer size for the line read
static MAX_LINE: u32 = 4096;

/// Unsupported Terminals that don't support RAW mode
static UNSUPPORTED_TERM: [&'static str; 3] = ["dumb","cons25","emacs"];

/// Key Strokes that rustyline should capture
const    NULL     : u8   = 0;     
const    CTRL_A   : u8   = 1;     
const    CTRL_B   : u8   = 2;     
const    CTRL_C   : u8   = 3;     
const    CTRL_D   : u8   = 4;     
const    CTRL_E   : u8   = 5;     
const    CTRL_F   : u8   = 6;     
const    CTRL_H   : u8   = 8;     
const    TAB      : u8   = 9;     
const    CTRL_K   : u8   = 11;    
const    CTRL_L   : u8   = 12;    
const    ENTER    : u8   = 13;    
const    CTRL_N   : u8   = 14;    
const    CTRL_P   : u8   = 16;    
const    CTRL_T   : u8   = 20;    
const    CTRL_U   : u8   = 21;    
const    CTRL_W   : u8   = 23;    
const    ESC      : u8   = 27;    
const    BACKSPACE: u8   = 127;    

/// Check to see if STDIN is a TTY
fn is_a_tty() -> bool {
    let isatty = unsafe { libc::isatty(libc::STDIN_FILENO as i32) } != 0;
    isatty
}

/// Check to see if the current `TERM` is unsupported
fn is_unsupported_term() -> bool {
    let term = std::env::var("TERM").ok().unwrap();
    let mut unsupported = false;
    for iter in &UNSUPPORTED_TERM {
        unsupported = term == *iter
    }
    unsupported
}

/// Enable raw mode for the TERM
fn enable_raw_mode() -> Result<termios::Termios, nix::Error> {
    if !is_a_tty() {
        Err(Sys(Errno::ENOTTY)) 
    } else {
        let original_term = try!(termios::tcgetattr(libc::STDIN_FILENO));
        let mut raw = original_term;
        raw.c_iflag = raw.c_iflag   & !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
        raw.c_oflag = raw.c_oflag   & !(OPOST);
        raw.c_cflag = raw.c_cflag   | (CS8);
        raw.c_lflag = raw.c_lflag   & !(ECHO | ICANON | IEXTEN | ISIG);
        raw.c_cc[VMIN] = 1;
        raw.c_cc[VTIME] = 0;
        try!(termios::tcsetattr(libc::STDIN_FILENO, termios::TCSAFLUSH, &raw));
        Ok(original_term)
    }
}

/// Disable Raw mode for the term
fn disable_raw_mode(original_termios: termios::Termios) -> Result<(), nix::Error> {
    try!(termios::tcsetattr(libc::STDIN_FILENO, termios::TCSAFLUSH, &original_termios));
    Ok(())
}

/// Handles reading and editting the readline buffer.
/// It will also handle special inputs in an appropriate fashion
/// (e.g., C-c will exit readline)
fn readline_edit() -> Result<String, io::Error> {
    let mut buffer = Vec::new();
    let mut input: [u8; 1] = [0];
    loop {
        let numread = io::stdin().read(&mut input).unwrap();
        match input[0] {
            CTRL_A => print!("Pressed C-a"),
            CTRL_B => print!("Pressed C-b"),
            CTRL_C => print!("Pressed C-c"),
            CTRL_D => print!("Pressed C-d"),
            CTRL_E => print!("Pressed C-e"),
            CTRL_F => print!("Pressed C-f"),
            CTRL_H => print!("Pressed C-h"),
            CTRL_K => print!("Pressed C-k"),
            CTRL_L => print!("Pressed C-l"),
            CTRL_N => print!("Pressed C-n"),
            CTRL_P => print!("Pressed C-p"),
            CTRL_T => print!("Pressed C-t"),
            CTRL_U => print!("Pressed C-u"),
            CTRL_W => print!("Pressed C-w"),
            ESC    => print!("Pressed esc") ,
            ENTER  => break,
            _      => { print!("{}", input[0]); io::stdout().flush(); }
        }
        buffer.push(input[0]);
    }
    Ok(String::from_utf8(buffer).unwrap())
}

/// Readline method that will enable RAW mode, call the ```readline_edit()```
/// method and disable raw mode
fn readline_raw() -> Result<String, io::Error> {
    if is_a_tty() {
        let original_termios = match enable_raw_mode() {
            Err(Sys(Errno::ENOTTY)) => return Err(Error::new(ErrorKind::Other, "Not a TTY")),
            Err(Sys(Errno::EBADF))  => return Err(Error::new(ErrorKind::Other, "Not a file descriptor")),
            Err(..)                 => return Err(Error::new(ErrorKind::Other, "Unknown Error")),
            Ok(term)                => term
        };

        let user_input = readline_edit();

        match disable_raw_mode(original_termios) {
            Err(..) => return Err(Error::new(ErrorKind::Other, "Failed to revert to original termios")),
            Ok(..)  => ()
        }

        user_input
    } else {

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_) => Ok(line),
            Err(e) => Err(e),
        }
    }
}

/// This is the only public library method that will be called by the end-user
pub fn readline(prompt: &'static str) -> Result<String, io::Error> {
    // Write prompt and flush it to stdout
    let mut stdout = io::stdout();
    try!(stdout.write(prompt.as_bytes()));
    try!(stdout.flush());

    if is_unsupported_term() {
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(_) => Ok(line),
            Err(e) => Err(e),
        }
    } else {
        readline_raw()
    }
}