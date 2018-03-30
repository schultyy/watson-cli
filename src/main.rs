extern crate clap;
extern crate flate2;
extern crate tar;

use std::fs::File;
use std::io::Error;
use std::error::Error as GeneralError;
use flate2::read::GzDecoder;
use tar::Archive;
use clap::{Arg, App};

fn decompress_file(filename: String) -> Result<(), Error> {
  let tar_gz = File::open(filename)?;
  let tar = GzDecoder::new(tar_gz);

  let mut archive = Archive::new(tar);
  archive.unpack(".").unwrap();
  Ok(())
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

  let filename = matches.value_of("INPUT").unwrap();
  println!("Using input file: {}", filename);

  match decompress_file(filename.to_string()) {
    Ok(()) => { },
    Err(err) => {
      println!("[ERR] Decompressing file: {}", err.description());
    }
  }
}
