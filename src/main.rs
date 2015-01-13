extern crate getopts;
extern crate time;

use std::os::{homedir, args};
use std::io::{File, SeekStyle, FileMode, FileAccess, FilePermission, USER_RWX};
use std::io::fs::{unlink, PathExtensions, mkdir};
use getopts::{reqopt, getopts, usage};
use time::{now_utc, Tm};

fn main() {
    let now = now_utc();
    let args = args();
    let home = homedir().unwrap();
    let base_dir = home.join(Path::new(".punch"));
    if !base_dir.exists() {
        mkdir(&base_dir, USER_RWX);
    }
    let timesheet_path = base_dir.join("timesheet");
    let mut working_state_path = base_dir.join("state");
    
    let mut timesheet = File::open_mode(&timesheet_path,
                                        FileMode::Append,
                                        FileAccess::Write).unwrap();
    timesheet.seek(0, SeekStyle::SeekEnd);

    match args.get(1) {
        None => panic!("No command given"),
        Some(command) => {
            match &command[] {
                "in" =>
                    punch_in(&now, &mut timesheet, &mut working_state_path),
                "out" =>
                    punch_out(&now, &mut timesheet, &mut working_state_path),
                _ => panic!("unknown command")
            }
        }
    }
}

fn punch_in(now: &Tm, timesheet: &mut File, state: &Path) {
    if currently_working(state) {
        panic!("You're already working");
    }
    writeln!(timesheet, "in: {}", now.rfc822());
    set_current_working_state(true, state);
}

fn punch_out(now: &Tm, timesheet: &mut File, state: &Path) {
    if !currently_working(state) {
        panic!("Can't punch out if you're not working!");
    }
    writeln!(timesheet, "out: {}", now.rfc822());
    set_current_working_state(false, state);
}

fn currently_working(p: &Path) -> bool {
    p.exists()
}

fn set_current_working_state(currently_working: bool, p: &Path) {
    if currently_working {
        File::create(p);
    } else {
        unlink(p);
    }
}
