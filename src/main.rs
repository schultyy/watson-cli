extern crate clap;

use clap::{Arg, App};


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
}
