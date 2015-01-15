#![allow(unstable)]

extern crate time;

use std::io::{File, SeekStyle, FileMode, FileAccess, USER_RWX, BufferedReader};
use std::io::fs::{unlink, PathExtensions, mkdir};
use std::os::{homedir, args};
use std::time::Duration;
use time::{now_utc, Tm, empty_tm, strptime};

fn main() {
    match args().get(1) {
        None => panic!("No command given"),
        Some(command) => {
            let mut time_clock = TimeClock::new();
            match &command[] {
                "in" => time_clock.punch_in(),
                "out" => time_clock.punch_out(),
                "status" => time_clock.status(),
                "report" => time_clock.report_daily_hours(),
                _ => panic!("unknown command")
            }
        }
    }
}

struct TimeClock {
    now: Tm,
    timesheet: File,
    currently_working: bool,
    state_path: Path
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
            currently_working: working_state_path.exists(),
            state_path: working_state_path,
            now: now
        }
    }

    // commands

    fn punch_in(&mut self) {
        if self.currently_working {
            panic!("You're already working");
        }
        self.timesheet.seek(0, SeekStyle::SeekEnd).unwrap();
        writeln!(&mut self.timesheet, "in: {}", self.now.rfc822()).unwrap();
        self.set_current_working_state(true);
    }

    fn punch_out(&mut self) {
        if !self.currently_working {
            panic!("Can't punch out if you're not working!");
        }
        self.timesheet.seek(0, SeekStyle::SeekEnd).unwrap();
        writeln!(&mut self.timesheet, "out: {}", self.now.rfc822()).unwrap();
        self.set_current_working_state(false);
    }

    fn status(&self) {
        if self.currently_working {
            println!("You're punched in")
        } else {
            println!("You're punched out")
        }
    }

    fn report_daily_hours(&mut self) {
        self.timesheet.seek(0, SeekStyle::SeekSet).unwrap();
        let mut buf = BufferedReader::new(File::open(self.timesheet.path()));
        let mut starting_time: Option<Tm> = None;
        let mut current_day = empty_tm();
        let mut time_worked_today = Duration::zero();
        for l in buf.lines() {
            // lines should alternate between starting with "in: " and "out: "
            let line = l.unwrap();
            if let Some(start) = starting_time {
                if !line.starts_with("out: ") {
                    panic!("Bad data in timesheet!");
                }
                let time_str = &line[5..];
                let end_time = parse_time(time_str);
                let time_passed = end_time.to_timespec() - start.to_timespec();
                time_worked_today = time_worked_today + time_passed;
                if !same_day(&current_day, &start) {
                    if time_worked_today > Duration::days(1) {
                        panic!("Worked more than 24 hours in a day!");
                    }
                    if !time_worked_today.is_zero() {
                        println!("{}: {}:{}",
                            start.strftime("%a, %d %b %Y").unwrap(),
                            time_worked_today.num_hours() ,
                            time_worked_today.num_minutes() % 60
                        );
                    }
                    current_day = start;
                }
                starting_time = None;
            } else {
                if !line.starts_with("in: ") {
                    panic!("Bad data in timesheet!");
                }
                let time_str = &line[4..];
                let t = parse_time(time_str);
                starting_time = Some(t);
            }
        }
    }

    // aux. methods

    fn set_current_working_state(&mut self, currently_working: bool) {
        self.currently_working = currently_working;
        if currently_working {
            File::create(&self.state_path).unwrap();
        } else {
            unlink(&self.state_path).unwrap();
        }
    }
}

fn parse_time(s: &str) -> Tm {
    strptime(s.slice_to(s.len() - 1), "%a, %d %b %Y %T %Z").unwrap()
}

fn same_day(t1: &Tm, t2: &Tm) -> bool {
    t1.tm_year == t2.tm_year && t1.tm_yday == t2.tm_yday
}
