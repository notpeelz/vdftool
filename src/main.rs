#![windows_subsystem = "console"]
#![deny(unsafe_op_in_unsafe_fn)]

mod cli;
mod json2vdf;
mod vdf2json;

use std::ffi::OsStr;

use eyre::Result;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn app(_entrypoint: bool) -> clap::Command {
  const PKG_NAME: &str = env!("CARGO_PKG_NAME");

  clap::Command::new(PKG_NAME)
    .about(
      "A set of tools for manipulating VDF files (Valve Data Format, aka KeyValues)",
    )
    .subcommand_required(true)
    .subcommand(vdf2json::command(false))
    .subcommand(json2vdf::command(false))
}

fn app_main(args: &clap::ArgMatches) -> Result<()> {
  match args.subcommand() {
    Some((vdf2json::NAME, args)) => vdf2json::main(args),
    Some((json2vdf::NAME, args)) => json2vdf::main(args),
    _ => unimplemented!(),
  }
}

#[cfg(not(debug_assertions))]
fn set_panic_handler() {
  std::panic::set_hook(Box::new(|panic_info| {
    eprintln!("{}", panic_info.to_string());
  }));
}

#[cfg(debug_assertions)]
fn set_panic_handler() {}

// XXX: we have to reset the SIGPIPE signal handler manually because
// the `unix_sigpipe` feature is unstable
#[cfg(unix)]
fn reset_sigpipe() {
  unsafe {
    libc::signal(libc::SIGPIPE, libc::SIG_DFL);
  }
}

#[cfg(not(unix))]
fn reset_sigpipe() {}

fn main() {
  reset_sigpipe();
  set_panic_handler();

  simple_eyre::install().unwrap();

  let Some(bin_name) = std::env::args()
    .next()
    .map(std::path::PathBuf::from)
    .and_then(|argv0| {
      argv0
        .file_name()
        .and_then(OsStr::to_str)
        .map(str::to_string)
    }) else {
      std::process::exit(1);
    };

  type CommandFn = fn(bool) -> clap::Command;
  type MainFn = fn(&clap::ArgMatches) -> Result<()>;

  let mut app = app as CommandFn;
  let mut main = app_main as MainFn;

  match &*bin_name {
    vdf2json::BIN_NAME => {
      app = vdf2json::command as CommandFn;
      main = vdf2json::main as MainFn;
    },
    json2vdf::BIN_NAME => {
      app = json2vdf::command as CommandFn;
      main = json2vdf::main as MainFn;
    },
    _ => {},
  }

  let args = app(true)
    .version(PKG_VERSION)
    .author(clap::crate_authors!("\n"))
    .disable_help_subcommand(true)
    .get_matches();

  if let Err(err) = main(&args) {
    eprintln!("{bin_name}: {err}");
  }
}
