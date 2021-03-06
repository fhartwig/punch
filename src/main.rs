extern crate time;

use std::fs::{File, create_dir, OpenOptions, remove_file};
use std::io::{Seek, SeekFrom, BufReader, BufRead, Lines, Write};
use std::io;
use std::path::{Path, PathBuf};
use std::env::{args, home_dir};
use std::fmt;
use std::process::exit;
use time::{Duration, now_utc, Tm, empty_tm, strptime};

fn main() {
    let result = match args().nth(1) {
        None => Err(PunchClockError::NoCommandGiven),
        Some(command) => {
            let mut time_clock = TimeClock::new().unwrap();
            match &command[..] {
                "in" => time_clock.punch_in(),
                "out" => time_clock.punch_out(),
                "status" => time_clock.status(),
                "report" => time_clock.report_daily_hours(),
                _ => Err(PunchClockError::UnknownCommand)
            }
        }
    };

    if let Err(e) = result {
        println!("Error: {}", e);
        exit(1);
    }
}

#[derive(Debug)]
enum PunchClockError {
    NoCommandGiven,
    UnknownCommand,
    AlreadyPunchedIn,
    AlreadyPunchedOut,
    CorruptedTimeSheet,
    IoError(io::Error),
}

impl fmt::Display for PunchClockError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use PunchClockError::*;
        fmt.write_str(
            match *self {
                NoCommandGiven => "No command given",
                UnknownCommand => "Unknown command",
                AlreadyPunchedIn => "You are already punched in",
                AlreadyPunchedOut => "You're not currently punched in",
                CorruptedTimeSheet => "Bad data in timesheet",
                IoError(_) => "IO error"
            }
        )
    }
}

impl From<io::Error> for PunchClockError {
    fn from(err: io::Error) -> PunchClockError {
        PunchClockError::IoError(err)
    }
}

type PunchClockResult<T> = Result<T, PunchClockError>;

struct TimeClock {
    now: Tm,
    timesheet: File,
    timesheet_path: PathBuf,
    currently_working: bool,
    state_path: PathBuf
}

impl TimeClock {
    fn new() -> PunchClockResult<TimeClock> {
        let now = now_utc();
        let home = home_dir().unwrap();
        let base_dir = home.join(Path::new(".punch"));
        let timesheet_path = base_dir.join("timesheet");
        let working_state_path = base_dir.join("state");
        if !path_exists(&base_dir) {
            try!(create_dir(&base_dir));
        }
        let timesheet = try!(OpenOptions::new().write(true).append(true)
                            .create(true).open(&timesheet_path));
        Ok(TimeClock {
            timesheet: timesheet,
            timesheet_path: timesheet_path,
            currently_working: path_exists(&working_state_path),
            state_path: working_state_path,
            now: now
        })
    }

    // commands

    fn punch_in(&mut self) -> PunchClockResult<()> {
        if self.currently_working {
            return Err(PunchClockError::AlreadyPunchedIn);
        }
        try!(self.timesheet.seek(SeekFrom::End(0)));
        writeln!(&mut self.timesheet, "in: {}", self.now.rfc822()).unwrap();
        self.set_current_working_state(true);
        Ok(())
    }

    fn punch_out(&mut self) -> PunchClockResult<()> {
        if !self.currently_working {
            return Err(PunchClockError::AlreadyPunchedOut);
        }
        try!(self.timesheet.seek(SeekFrom::End(0)));
        try!(writeln!(&mut self.timesheet, "out: {}", self.now.rfc822()));
        self.set_current_working_state(false);
        Ok(())
    }

    fn status(&self) -> PunchClockResult<()> {
        if self.currently_working {
            println!("You're punched in");
        } else {
            println!("You're punched out");
        }
        Ok(())
    }

    fn report_daily_hours(&mut self) -> PunchClockResult<()> {
        try!(self.timesheet.seek(SeekFrom::Start(0)));
        let buf = BufReader::new(try!(File::open(&self.timesheet_path)));
        let mut current_day = empty_tm();
        let mut time_worked_today = Duration::zero();

        for interval in IntervalIter::from_lines(buf.lines()) {
            let (start, end) = try!(interval);
            if !same_day(&start, &current_day) {
                if !time_worked_today.is_zero() {
                    print_time_worked(&time_worked_today, &current_day);
                }
                current_day = start;
                time_worked_today = Duration::zero();
            }
            time_worked_today =
                time_worked_today + (end.to_timespec() - start.to_timespec());
        }

        if !time_worked_today.is_zero() {
            print_time_worked(&time_worked_today, &current_day);
        }
        Ok(())
    }

    // aux. methods

    fn set_current_working_state(&mut self, currently_working: bool) {
        self.currently_working = currently_working;
        if currently_working {
            File::create(&self.state_path).unwrap();
        } else {
            remove_file(&self.state_path).unwrap();
        }
    }
}

struct IntervalIter {
    lines: Lines<BufReader<File>>
}

impl IntervalIter {
    fn from_lines(lines: Lines<BufReader<File>>) -> IntervalIter {
        IntervalIter {lines: lines}
    }
}

impl Iterator for IntervalIter {
    type Item = PunchClockResult<(Tm, Tm)>;
    fn next(&mut self) -> Option<PunchClockResult<(Tm, Tm)>> {

        // helper function to make error handling a bit nicer
        fn inner_unwrap<T>(x: Option<io::Result<T>>)
                -> PunchClockResult<Option<T>> {
            match x {
                None => Ok(None),
                Some(Ok(inner)) => Ok(Some(inner)),
                Some(Err(e)) => Err(PunchClockError::IoError(e))
            }
        }

        let line_1 = match inner_unwrap(self.lines.next()) {
            Ok(l) => l,
            Err(e) => return Some(Err(e))
        };
        let line_2 = match inner_unwrap(self.lines.next()) {
            Ok(l) => l,
            Err(e) => return Some(Err(e))
        };

        match (line_1, line_2) {
            (None, None) => None,
            (Some(start_line), o_end_line) => {
                if !start_line.starts_with("in: ") {
                    return Some(Err(PunchClockError::CorruptedTimeSheet));
                }
                let start = parse_time(&start_line[4..]);
                let end = match o_end_line {
                    None => now_utc(),
                    Some(end_line) => {
                        if !end_line.starts_with("out: ") {
                            return Some(Err(PunchClockError::CorruptedTimeSheet));
                        }
                        parse_time(&end_line[5..])
                    },
                };
                Some(Ok((start, end)))
            },
            _ => unreachable!() // (None, Some(l)) should not happen
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

fn parse_time(s: &str) -> Tm {
    strptime(&s[..s.len() - 1], "%a, %d %b %Y %T %Z").unwrap()
}

fn same_day(t1: &Tm, t2: &Tm) -> bool {
    t1.tm_year == t2.tm_year &&
    t1.tm_mon == t2.tm_mon &&
    t1.tm_mday == t2.tm_mday
}

fn print_time_worked(t: &Duration, day: &Tm) {
    println!("{}: {:>2}:{:02}",
        day.strftime("%a, %d %b %Y").unwrap(),
        t.num_hours() ,
        t.num_minutes() % 60
    );
}

fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    ::std::fs::metadata(path).is_ok()
}
