extern crate getopts;
extern crate time;

use std::os::{homedir, args};
use std::io::{File, SeekStyle, FileMode, FileAccess};
use getopts::{reqopt, getopts, usage};
use time::{now_utc, Tm};

fn main() {
    let args = args();
    let home = homedir().unwrap();
    let timesheet_path = home.join(Path::new(".punch"));

    let now = now_utc();

    let mut timesheet = File::open_mode(&timesheet_path,
                                        FileMode::Append,
                                        FileAccess::Write).unwrap();
    timesheet.seek(0, SeekStyle::SeekEnd);

    match args.get(1) {
        None => panic!("No command given"),
        Some(command) => {
            match &command[] {
                "in" => punch_in(&now, &mut timesheet),
                "out" => punch_out(&now, &mut timesheet),
                _ => panic!("unknown command")
            }
        }
    }
}

fn punch_in(now: &Tm, timesheet: &mut File) {
    writeln!(timesheet, "in: {}", now.rfc822());
    // TODO: make sure we're not already punched in
}

fn punch_out(now: &Tm, timesheet: &mut File) {
    writeln!(timesheet, "out: {}", now.rfc822());
    // TODO: make sure we're punched in
}
