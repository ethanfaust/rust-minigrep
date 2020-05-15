use std::env;
use std::fmt::{Display, Formatter, Error};
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use regex::Regex;

fn print_usage() {
    let args: Vec<String> = env::args().collect();
    let prog_name = &args[0];
    eprintln!("usage: {} [options] query file", prog_name);
    eprintln!("");
    eprintln!("file: file path");
    eprintln!("query: search string as regex");
    eprintln!("options:");
    eprintln!("    -v: invert match: print lines that do not match instead");
    eprintln!("    -g: dump regex capture groups");
    eprintln!("");
    eprintln!("Author: Ethan Faust");
    eprintln!("");
}

struct MinigrepOptions {
    filename: String,
    query: String,
    invert_match: bool,
    dump_capture_groups: bool,
}

fn parse_args(args: &[String]) -> Result<MinigrepOptions, &str> {
    let arg_count = args.len();
    if arg_count < 3 {
        return Err("not enough arguments");
    }

    let mut invert_match: bool = false;
    let mut dump_capture_groups: bool = false;
    for arg_index in 1..(arg_count - 2) {
        let arg = &args[arg_index];
        if arg == "-v" {
            invert_match = true;
        }
        if arg == "-g" {
            dump_capture_groups = true;
        }
    }

    let query = &args[arg_count - 2];
    let filename = &args[arg_count - 1];

    Ok(MinigrepOptions {
        filename: filename.to_string(),
        query: query.to_string(),
        invert_match: invert_match,
        dump_capture_groups: dump_capture_groups,
    })
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = match parse_args(&args) {
        Err(_e) => {
            print_usage();
            std::process::exit(1);
        }
        Ok(opt) => opt
    };
    run(&options);
}

fn match_line(_options: &MinigrepOptions, re: &Regex, line: &str) -> bool {
    let is_match = re.is_match(line);
    return is_match;
}

fn output_line(options: &MinigrepOptions, re: &Regex, line: &str, is_match: bool) {
    let mut should_write = is_match;

    if options.invert_match {
        should_write = !is_match;
    }
    if !should_write {
        return;
    }

    if options.dump_capture_groups {
        write_capture_groups(options, re, line);
    } else {
        normal_output(options, line);
    }
}

fn normal_output(_options: &MinigrepOptions, line: &str) {
    println!("{}", &line);
}

fn write_capture_groups(_options: &MinigrepOptions, re: &Regex, line: &str) {
    let captures = re.captures(line);
    if captures.is_none() {
        return;
    }
    let captures = captures.unwrap();
    let matches : Vec<&str> = captures.iter()
        .map(|c| c.map_or("", |m| m.as_str()))
        .collect();
    let capture_vec = CaptureGroupVec(matches);
    println!("{}", capture_vec);
}

struct CaptureGroupVec<'a>(Vec<& 'a str>);
impl Display for CaptureGroupVec<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut comma_separated = String::new();
        for capture in &self.0[1..self.0.len() - 1] {
            comma_separated.push_str(&capture);
            comma_separated.push_str(",");
        }
        comma_separated.push_str(&self.0[self.0.len() - 1]);
        write!(f, "{}", comma_separated)
    }
}

fn run(options: &MinigrepOptions) {
    let path = Path::new(&options.filename);
    let path_display = path.display();
    let file = File::open(&path).unwrap_or_else(|e| {
        eprintln!("couldn't open {}: {}", path_display, e);
        std::process::exit(1);
    });
    let reader = BufReader::new(file);

    let re = Regex::new(&options.query).unwrap_or_else(|e| {
        eprintln!("error parsing pattern {}: {}", &options.query, e);
        std::process::exit(1);
    });

    for line in reader.lines() {
        let line = line.unwrap_or_else(|e| {
            eprintln!("error reading file: {}", e);
            std::process::exit(1);
        });
        let is_match = match_line(options, &re, &line);
        output_line(options, &re, &line, is_match);
    }
}
