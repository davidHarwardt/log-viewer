use std::{path::Path, io::{BufReader, Seek, SeekFrom, BufRead, stdout}, fs::{File, Metadata}, time::Duration};


use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
};
use notify::{Watcher, event::ModifyKind::Any, EventKind::Modify};

const MSG: &str = "log viewer
press 'q' or 'escape' to exit";
const MSG_LEN: usize = 29;

const CLEAR_MSG: &str = "\n<cleared logfile>\n";

fn next_block(v: &str) -> usize {
    if v.starts_with("[") {
        let mut depth = 1;
        let end = v.chars().skip(1).position(|c| match c {
            '[' => { depth += 1; false },
            ']' => { depth -= 1; depth == 0 },
            _ => { false },
        }).map(|v| v + 2).unwrap_or(v.len());

        match &v[0..end] {
            v @ "[debug]" | v @ "[DEBUG]" => print!("{}", v.blue()),
            v @ "[info]" | v @ "[INFO]" => print!("{}", v.dark_green()),
            v @ "[warn]" | v @ "[WARN]" => print!("{}", v.yellow()),
            v @ "[error]" | v @ "[ERROR]" => print!("{}", v.red()),
            v => print!("{}", v.dark_grey()),
        }

        end
    } else {
        let end = v.chars().position(|c| c == '[').unwrap_or(v.len());
        print!("{}", &v[0..end]);

        end
    }
}

fn print_line(line: &str) {
    let mut current = &line[..];
    while current.len() != 0 {
        let len = next_block(current);
        current = &current[len..];
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 { println!("please provide a filename"); std::process::exit(1) }

    let path = Path::new(&args[1]);


    let file = File::open(path).expect("could not open file");
    let meta = file.metadata().expect("could not get file metadata");
    let mut pos = meta.len();

    let mut watcher = notify::PollWatcher::new(move |res: Result<notify::Event, notify::Error>| {
        match res {
            Ok(v) => {
                let path = v.paths.first().unwrap();
                match v.kind {
                    Modify(_) => {
                        let file = File::open(path).expect("could not open file");
                        let meta = file.metadata().expect("could not get file metadata");
                        if meta.len() < pos {
                            println!("{}", CLEAR_MSG.dark_grey());
                            pos = 0;
                        }
                        let mut reader = BufReader::new(file);
                        reader.seek(SeekFrom::Start(pos)).unwrap();

                        let mut rem;
                        let mut line = String::new();

                        loop {
                            rem = reader.read_line(&mut line).unwrap();
                            if rem == 0 { break }
                            pos += rem as u64;

                            print_line(&line);
                            line.clear();
                        }
                        
                    },
                    v => { println!("{v:?}") },
                }
            },
            Err(err) => println!("{err:?}"),
        }
    }, notify::Config::default().with_poll_interval(Duration::from_millis(100))).expect("could not create watcher");

    watcher.watch(path, notify::RecursiveMode::NonRecursive).expect("could not watch file");

    println!("{}", MSG.bold());
    let file_name = format!("watching '{}'", path.display());
    println!("{}", file_name);
    println!("{}", "-".repeat(file_name.len().max(MSG_LEN)));

    loop {
        match event::read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        std::process::exit(0);
                    },
                    _ => {},
                }
            },
            _ => {},
        }
    }
}

