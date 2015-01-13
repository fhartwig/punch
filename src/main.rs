#![allow(unstable)]

extern crate time;

use std::os::{homedir, args};
use std::io::{File, SeekStyle, FileMode, FileAccess, USER_RWX};
use std::io::fs::{unlink, PathExtensions, mkdir};
use time::{now_utc, Tm};

fn main() {
    let args = args();
    match args.get(1) {
        None => panic!("No command given"),
        Some(command) => {
            let mut time_clock = TimeClock::new();
            match &command[] {
                "in" => time_clock.punch_in(),
                "out" => time_clock.punch_out(),
                _ => panic!("unknown command")
            }
        }
    }
}

struct TimeClock {
    now: Tm,
    timesheet: File,
    state_path: Path // TODO: maybe we can make this a bool that we
                           // initalise with the TimeClock
                           // and write back when we destroy the timeclock
                            // object
}

impl TimeClock {
    fn new() -> TimeClock {
        let now = now_utc();
        let home = homedir().unwrap();
        let base_dir = home.join(Path::new(".punch"));
        let timesheet_path = base_dir.join("timesheet");
        let working_state_path = base_dir.join("state");
        if !base_dir.exists() {
            mkdir(&base_dir, USER_RWX).unwrap();
        }
        let timesheet = File::open_mode(&timesheet_path,
                                            FileMode::Append,
                                            FileAccess::Write).unwrap();
        TimeClock {
            timesheet: timesheet,
            state_path: working_state_path,
            now: now
        }
    }

    fn punch_in(&mut self) {
        if self.currently_working() {
            panic!("You're already working");
        }
        self.timesheet.seek(0, SeekStyle::SeekEnd).unwrap();
        writeln!(&mut self.timesheet, "in: {}", self.now.rfc822()).unwrap();
        self.set_current_working_state(true);
    }

    fn punch_out(&mut self) {
        if !self.currently_working() {
            panic!("Can't punch out if you're not working!");
        }
        self.timesheet.seek(0, SeekStyle::SeekEnd).unwrap();
        writeln!(&mut self.timesheet, "out: {}", self.now.rfc822()).unwrap();
        self.set_current_working_state(false);
    }


    fn set_current_working_state(&self, currently_working: bool) {
        if currently_working {
            File::create(&self.state_path).unwrap();
        } else {
            unlink(&self.state_path).unwrap();
        }
    }

    fn currently_working(&self) -> bool {
        self.state_path.exists()
    }
}
