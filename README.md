# Punch - a simple time tracking tool

Punch is a simple time tracking tool written in Rust.

## Installation

To install `Punch` you will need the rustc compiler and the cargo package
manager. You can find these [here](http://www.rust-lang.org/install.html).
You should be able to use Rust 1.0 or any later version.

Then you can build the executable by calling

    cargo build --release

from the repository root and install it by copying the executable at
`target/release/punch` into some directory on your path.

## Usage

Punch implements four commands:

- `punch in` marks you as working
- `punch out` says that you have finished working
- `punch status` tells you whether you are currently punched in
- `punch report` prints a report of the time you worked each day to standard out
