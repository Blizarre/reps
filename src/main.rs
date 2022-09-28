#[macro_use]
extern crate clap;
extern crate termion;

use std::io::{stdout, Bytes, Read};
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use std::process::exit;
use termion::color::Color;
use termion::raw::IntoRawMode;
use termion::{async_stdin, color, AsyncReader};

const DURATION_500_MILLISECONDS: Duration = Duration::from_millis(500);
const DURATION_1_SECOND: Duration = Duration::from_millis(1000);

/// Display a single message on the screen, starting from the upper left.
/// It will clear the screen and reset the text color at the end.
fn print_message(message: &str) {
    println!(
        "{}{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        message,
        termion::color::Fg(color::Reset)
    );
}

/// Consume all the keys from the standard input. Is in charge of detecting if the user request to
/// the program (ESC or Ctrl-C).
///
/// # Returns
/// - if no keys were pressed, Ok(None)
/// - If any key was pressed that should not exit the program, Ok(Some)
/// - If a key was pressed that should stop the program (ESC, Ctrl-C), Err("Exiting")
/// - If an error occurred, Err(<error message)
fn consume_all_keystrokes(stdin: &mut Bytes<AsyncReader>) -> Result<Option<()>, String> {
    let mut return_value = Ok(None);

    loop {
        match stdin.next() {
            Some(e) => {
                match e {
                    Ok(27) => return Err("Exiting".to_string()), // ESC
                    Ok(3) => return Err("Exiting".to_string()),  // Ctrl-C
                    Err(e) => return Err(format!("Error: {}", e)),

                    Ok(_) => return_value = Ok(Some(())),
                }
            }
            None => return return_value,
        }
    }
}

/// This method will check if keys were pressed since the last time it was called and will pause
/// if that's the case until the user press another key.
/// It will also forward requests to stop the program (and Errors).
fn handle_pause(stdin: &mut Bytes<AsyncReader>) -> Result<(), String> {
    if consume_all_keystrokes(stdin)?.is_some() {
        print_message("PAUSE");
        while consume_all_keystrokes(stdin)?.is_none() {
            thread::sleep(DURATION_500_MILLISECONDS)
        }
    }
    Ok(())
}

/// Display a countdown with the specific label and colors.
/// It will periodically check if the user entered any input and forward requests to stop the
/// program as errors.
fn countdown(
    stdin: &mut Bytes<AsyncReader>,
    label: &str,
    count: u32,
    color: &dyn Color,
) -> Result<(), String> {
    for sec in (1..=count).rev() {
        print_message(
            format!(
                "{}{}\n{}{}s",
                termion::color::Fg(color),
                label,
                termion::color::Fg(color::Blue),
                sec
            )
            .as_str(),
        );
        thread::sleep(DURATION_1_SECOND);
        handle_pause(stdin.by_ref())?;
    }
    Ok(())
}

struct Options {
    num_reps: u32,
    rep_time: u32,
    relax_time: u32,
}

fn parse_args() -> Result<Options, clap::Error> {
    let matches = App::new("Reps")
        .version("0.1.0")
        .author("Simon M. <git@simon.marache.net>")
        .arg(Arg::with_name("num_reps").required(true))
        .arg(Arg::with_name("rep_time").required(true))
        .arg(Arg::with_name("relax_time").required(true))
        .get_matches();

    let num_reps = value_t!(matches.value_of("num_reps"), u32)?;
    let rep_time = value_t!(matches.value_of("rep_time"), u32)?;
    let relax_time = value_t!(matches.value_of("relax_time"), u32)?;
    Ok(Options {
        num_reps,
        rep_time,
        relax_time,
    })
}

fn main() {
    let opts = match parse_args() {
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
        Ok(r) => r,
    };

    let stdin = async_stdin().bytes();
    let stdout = stdout();

    // We need to be able to asynchronously check for input from the user, bypassing all the caching
    // and Control keys handling provided by the terminal. The only way is to put the terminal in
    // raw mode
    let stdout = stdout.lock().into_raw_mode().unwrap();
    println!("{}{}", termion::clear::All, termion::cursor::Hide);

    let result = start_reps(stdin, opts.num_reps, opts.rep_time, opts.relax_time);

    // Bring the cursor back to a usable state
    println!(
        "{}{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Show,
        termion::color::Fg(color::Reset)
    );

    stdout
        .suspend_raw_mode()
        .expect("Error when reverting suspend mode");

    let _ = result
        .map(|_result| println!("done"))
        .map_err(|error| println!("{}", error));
}

/// Display the countdowns using the values provided by the user
fn start_reps(
    mut stdin: Bytes<AsyncReader>,
    reps: u32,
    time: u32,
    time_between_reps: u32,
) -> Result<(), String> {
    countdown(&mut stdin, "Starting in", 3, &color::Blue)?;
    for rep in 1..=reps {
        countdown(
            &mut stdin,
            format!("Rep {}/{}", rep, reps).as_str(),
            time,
            &color::Red,
        )?;
        countdown(&mut stdin, "Relax!", time_between_reps, &color::Green)?;
    }
    Ok(())
}
