#[macro_use]
extern crate clap;
extern crate termion;

use std::io::{stdout, Bytes, Read};
use std::thread;
use std::time::Duration;

use clap::{App, Arg};
use termion::raw::IntoRawMode;
use termion::{async_stdin, color, AsyncReader};

const DURATION_500_MILLISECONDS: Duration = Duration::from_millis(500);
const DURATION_1_SECOND: Duration = Duration::from_millis(1000);

fn print_message(message: &str) {
    println!(
        "{}{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        message,
        termion::color::Fg(color::Reset)
    );
}

fn consume_all(stdin: &mut Bytes<AsyncReader>) -> Result<Option<()>, String> {
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

fn handle_pause(stdin: &mut Bytes<AsyncReader>) -> Result<(), String> {
    if consume_all(stdin)?.is_some() {
        print_message("PAUSE");
        while consume_all(stdin)?.is_none() {
            thread::sleep(DURATION_500_MILLISECONDS)
        }
    }
    Ok(())
}

fn main() {
    let matches = App::new("Reps")
        .version("0.1.0")
        .author("Simon M. <git@simon.marache.net>")
        .arg(Arg::with_name("num_reps").required(true))
        .arg(Arg::with_name("rep_time").required(true))
        .arg(Arg::with_name("relax_time").required(true))
        .get_matches();

    let stdin = async_stdin().bytes();
    let _stdout = stdout();
    let stdout = _stdout.lock().into_raw_mode().unwrap();

    let num_reps =
        value_t!(matches.value_of("num_reps"), u32).expect("Invalid num_reps, must be a positive number");
    let rep_time =
        value_t!(matches.value_of("rep_time"), u32).expect("Invalid rep_time, must be a positive number");
    let relax_time = value_t!(matches.value_of("relax_time"), u32)
        .expect("Invalid relax_time, must be a positive number");

    println!("{}{}", termion::clear::All, termion::cursor::Hide);

    let result = start_reps(stdin, num_reps, rep_time, relax_time);

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

fn start_reps(
    mut stdin: Bytes<AsyncReader>,
    reps: u32,
    time: u32,
    time_between_reps: u32,
) -> Result<(), String> {
    for sec in (1..=3).rev() {
        print_message(
            format!(
                "{}Starting in\n{}{}s",
                termion::color::Fg(color::Red),
                termion::color::Fg(color::Blue),
                sec
            )
            .as_str(),
        );
        thread::sleep(DURATION_1_SECOND);
        handle_pause(stdin.by_ref())?;
    }
    for rep in 1..=reps {
        for sec in (1..=time).rev() {
            print_message(
                format!(
                    "{}Rep #{}/{}\n{}{}s",
                    termion::color::Fg(color::Red),
                    rep,
                    reps,
                    termion::color::Fg(color::Blue),
                    sec
                )
                .as_str(),
            );
            thread::sleep(DURATION_1_SECOND);
            handle_pause(stdin.by_ref())?;
        }
        for sec in (1..=time_between_reps).rev() {
            print_message(
                format!(
                    "{}Relax\n{}{}s",
                    termion::color::Fg(color::Green),
                    termion::color::Fg(color::Blue),
                    sec
                )
                .as_str(),
            );
            thread::sleep(DURATION_1_SECOND);
            handle_pause(stdin.by_ref())?;
        }
    }
    Ok(())
}
