/*!
# set-filter

Set-filter performs fast, in-memory operations over sets represented as
text streams.

I know, you jerks are gonna say, "but how fast is it?" The answer is it's
faster than the one-off powershell script I wrote that does the same thing,
both in the amount of time required to rewrite it (read: zero) and the time
required to run it. Feel free to compare it against your own one-off shell
script if you want damned benchmarks.

```
set-filter 0.1.0

USAGE:
    sf.exe [FLAGS] [path] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -r, --reverse    take only repeated items
    -V, --version    Prints version information

ARGS:
    <path>    the base set (optional; input may be taken from stdin instead)

SUBCOMMANDS:
    diff         set difference
    help         Prints this message or the help of the given subcommand(s)
    intersect    set intersection
```
*/

use std::{
    fmt::Display,
    fs,
    io::{self, Read, Write},
};

use hashbrown::HashSet;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
struct Opts {
    /// the base set (optional; input may be taken from stdin instead)
    path: Option<String>,
    /// take only repeated items
    #[structopt(short, long)]
    reverse: bool,

    #[structopt(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Debug, StructOpt)]
enum Command {
    /// set difference
    Diff(Diff),
    /// set intersection
    Intersect(Intersect),
}

#[derive(Clone, Debug, StructOpt)]
struct Diff {
    pub path: String,
}

#[derive(Clone, Debug, StructOpt)]
struct Intersect {
    pub path: String,
}

trait WithOpts {
    fn run<E: std::error::Error>(&self, ex: impl FnOnce(&Self) -> Result<(), E>);
}

impl<T: StructOpt> WithOpts for T {
    fn run<E: std::error::Error>(&self, ex: impl FnOnce(&T) -> Result<(), E>) {
        if let Err(e) = ex(self) {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    Opts::from_args().run(run);
}

fn run(opts: &Opts) -> io::Result<()> {
    match &opts.command {
        Some(Command::Diff(Diff { path })) => print_difference(opts, path),
        Some(Command::Intersect(Intersect { path })) => print_intersection(opts, &path),
        None => print_unique(opts),
    }
}

fn print_difference(opts: &Opts, path: &str) -> io::Result<()> {
    let text = read_text(opts)?;
    let compare = fs::read_to_string(path)?;
    let a: HashSet<_> = text.lines().collect();
    let b: HashSet<_> = compare.lines().collect();
    let difference = a.difference(&b);
    format(difference)
}

fn print_intersection(opts: &Opts, path: &str) -> io::Result<()> {
    let text = read_text(opts)?;
    let compare = fs::read_to_string(path)?;
    let a: HashSet<_> = text.lines().collect();
    let b: HashSet<_> = compare.lines().collect();
    let intersection = a.intersection(&b);
    format(intersection)
}

fn print_unique(opts: &Opts) -> io::Result<()> {
    let text = read_text(opts)?;
    let mut a = HashSet::new();
    if opts.reverse {
        let mut b = HashSet::new();
        let repeated = text
            .lines()
            .filter(|&value| !a.insert(value) && b.insert(value));
        format(repeated)
    } else {
        let unique = text.lines().filter(|&value| a.insert(value));
        format(unique)
    }
}

#[inline]
fn format(values: impl IntoIterator<Item = impl Display>) -> io::Result<()> {
    let handle = io::stdout();
    let mut lock = handle.lock();
    values
        .into_iter()
        .try_for_each(|value| writeln!(lock, "{}", value))
}

fn read_text(opts: &Opts) -> io::Result<String> {
    match &opts.path {
        Some(path) => fs::read_to_string(path),
        None => {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            Ok(buf)
        }
    }
}
