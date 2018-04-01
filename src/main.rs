extern crate clap;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
extern crate notify;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

use clap::{Arg, App, SubCommand};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::io::{self, Error};
use std::sync::mpsc::channel;
use std::time::Duration;
use futures::{Future, Stream};
use tokio_core::reactor::Core;
use hyper::{Client, Method, Request};
use hyper::header::{ContentLength, ContentType};
use serde_json::Value;
use notify::{RecommendedWatcher, Watcher, RecursiveMode};

#[derive(Serialize)]
pub struct LogFile {
  pub content: String,
  pub name: String
}

#[derive(Serialize)]
pub struct NewAnalyzer {
  analyzer: String
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

fn index_file(full_path: &str, server: &str) {
  let file_content = read_file(full_path.to_string()).expect("Cannot read file");
  let filename = get_filename(full_path).expect(&format!("Cannot get filename for {}", full_path));
  let json_content = produce_json_payload(filename, file_content);

  let mut core = Core::new().expect("Cannot build event loop");
  let client = Client::new(&core.handle());
  let uri = format!("{}/file", server).parse().expect("Expected valid server");
  let mut request = Request::new(Method::Post, uri);
  request.headers_mut().set(ContentType::json());
  request.headers_mut().set(ContentLength(json_content.len() as u64));
  request.set_body(json_content);
  let post = client.request(request).and_then(|response| {
    println!("POST: {}", response.status());
    response.body().concat2().and_then(move |body| {
      let v: Value = serde_json::from_slice(&body).map_err(|e| {
          io::Error::new(
              io::ErrorKind::Other,
              e
          )
      })?;
      println!("{}", v["id"].as_str().expect("ID has to be of data type string"));
      Ok(())
    })
  });
  core.run(post).unwrap();
}

fn add_analyzer(new_analyzer: &str, server: &str) {
  let json_content = json!(NewAnalyzer { analyzer: new_analyzer.to_string() }).to_string();

  let mut core = Core::new().expect("Cannot build event loop");
  let client = Client::new(&core.handle());
  let uri = format!("{}/analyzer", server).parse().expect("Expected valid server");
  let mut request = Request::new(Method::Post, uri);
  request.headers_mut().set(ContentType::json());
  request.headers_mut().set(ContentLength(json_content.len() as u64));
  request.set_body(json_content);
  let post = client.request(request).and_then(|response| {
    println!("POST: {}", response.status());
    response.body().concat2().and_then(move |body| {
      let v: Value = serde_json::from_slice(&body).map_err(|e| {
          io::Error::new(
              io::ErrorKind::Other,
              e
          )
      })?;
      println!("{}", v);
      Ok(())
    })
  });
  core.run(post).unwrap();
}

fn watch(directory: &str) -> notify::Result<()> {
  let (tx, rx) = channel();

  let mut watcher: RecommendedWatcher = try!(Watcher::new(tx, Duration::from_secs(2)));
  try!(watcher.watch(directory, RecursiveMode::Recursive));

  loop {
    match rx.recv() {
      Ok(event) => println!("{:?}", event),
      Err(e) => println!("watch error: {:?}", e),
    }
  }
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
                        .subcommand(SubCommand::with_name("index")
                              .about("Indexes a new file")
                              .arg(Arg::with_name("INPUT")
                                    .help("Sets the input file to use")
                                    .required(true)
                                    .index(1)))
                        .subcommand(SubCommand::with_name("add_analyzer")
                              .about("Adds a new analyzer")
                              .arg(Arg::with_name("ANALYZER")
                                    .help("Sets the new analyzer to add")
                                    .required(true)
                                    .index(1)))
                        .subcommand(SubCommand::with_name("watch")
                              .about("Watches a specific directory")
                              .arg(Arg::with_name("DIRECTORY")
                                    .help("Specifies the directory to watch")
                                    .required(true)
                                    .index(1)))
                        .get_matches();

  let server = matches.value_of("server").unwrap_or("http://localhost:8000");
  println!("Value for server: {}", server);

  if let Some(index_command) = matches.subcommand_matches("index") {
    let full_path = index_command.value_of("INPUT").unwrap();
    println!("Using input file: {}", full_path);
    index_file(full_path, server);
  }

  if let Some(add_analyzer_command) = matches.subcommand_matches("add_analyzer") {
    let new_analyzer = add_analyzer_command.value_of("ANALYZER").unwrap();
    println!("Adding analyzer: {}", new_analyzer);
    add_analyzer(new_analyzer, server);
  }

  if let Some(directory_watch_command) = matches.subcommand_matches("watch") {
    let watch_path = directory_watch_command.value_of("DIRECTORY").unwrap();
    println!("Watching directory {}", watch_path);
    if let Err(e) = watch(watch_path) {
      println!("error: {:?}", e)
    }
  }
}
