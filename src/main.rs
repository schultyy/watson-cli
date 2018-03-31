extern crate clap;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

use clap::{Arg, App};
use std::fs::File;
use std::io::prelude::*;
use std::io::Error;
use std::path::Path;
use std::ffi::OsStr;

#[derive(Serialize)]
pub struct LogFile {
  pub content: String,
  pub name: String
}

fn read_file(file_path: String) -> Result<String, Error> {
  let mut file = File::open(file_path)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  Ok(contents)
}

fn get_filename(file_path: &str) -> Option<String> {
  Path::new(file_path).file_name()
                        .and_then(|filename| filename.to_str())
                        .map(|s| s.to_string())
}

fn produce_json_payload(filename: String, contents: String) -> String {
  json!(LogFile{
    name: filename,
    content: contents
  }).to_string()
}

fn main() {
  let matches = App::new("Watson CLI")
                        .version("1.0")
                        .author("Jan Schulte <hello@unexpected-co.de>")
                        .about("Command line interface for watsond")
                        .arg(Arg::with_name("server")
                              .short("s")
                              .long("server")
                              .value_name("SERVER")
                              .help("Sets a custom server")
                              .takes_value(true))
                        .arg(Arg::with_name("INPUT")
                              .help("Sets the input file to use")
                              .required(true)
                              .index(1))
                        .get_matches();

  let server = matches.value_of("server").unwrap_or("http://localhost:8000");
  println!("Value for server: {}", server);

  let full_path = matches.value_of("INPUT").unwrap();
  println!("Using input file: {}", full_path);

  let file_content = read_file(full_path.to_string()).expect("Cannot read file");
  let filename = get_filename(full_path).expect(&format!("Cannot get filename for {}", full_path));
  let json_content = produce_json_payload(filename, file_content);

  println!("{}", json_content);
}
