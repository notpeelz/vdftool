use std::io::Write;

use eyre::{bail, Result};

use crate::cli::{ArgMatchesExt, CommandExt};

pub(crate) const NAME: &str = "vdf2json";

#[cfg(not(windows))]
pub(crate) const BIN_NAME: &str = "vdf2json";

#[cfg(windows)]
pub(crate) const BIN_NAME: &str = "vdf2json.exe";

pub(crate) fn command(entrypoint: bool) -> clap::Command {
  clap::Command::new(NAME)
    .about(if entrypoint {
      "Convert VDF (Valve Data Format, aka KeyValues) to JSON"
    } else {
      "Convert VDF to JSON"
    })
    .with_file_arg()
    .arg(
      clap::Arg::new("top-level")
        .short('t')
        .long("top-level")
        .help("Preserves the top level VDF key")
        .action(clap::ArgAction::SetTrue)
        .num_args(0),
    )
    .arg(
      clap::Arg::new("pretty")
        .short('p')
        .long("pretty")
        .help("Output pretty-printed JSON")
        .action(clap::ArgAction::SetTrue)
        .num_args(0),
    )
}

pub(crate) fn main(args: &clap::ArgMatches) -> Result<()> {
  let (stream, err_builder) = args.get_input_file()?;

  let data = std::io::read_to_string(stream)
    .map_err(|err| err_builder.format(format_args!("failed to read: {err}")))?;

  if !data.as_bytes().iter().any(|b| {
    char::from_u32(*b as u32).is_some_and(|c| !['\r', '\n'].contains(&c))
  }) {
    bail!(err_builder.format(format_args!("no data")));
  }

  let json = {
    let (json, key) = keyvalues_serde::from_str_with_key::<
      serde_json::Map<String, serde_json::Value>,
    >(&data)
    .map_err(|err| {
      err_builder.format(format_args!("failed to deserialize VDF: {err}"))
    })?;

    macro_rules! json_to_string {
      ($e:expr) => {
        if args.get_flag("pretty") {
          serde_json::to_string_pretty($e)
        } else {
          serde_json::to_string($e)
        }
      };
    }

    if args.get_flag("top-level") {
      json_to_string!(&serde_json::json!({ key: json }))
    } else {
      json_to_string!(&json)
    }
    .map_err(|err| {
      err_builder.format(format_args!("failed to serialize to JSON: {err}"))
    })?
  };

  std::io::stdout()
    .write_all(json.as_bytes())
    .map_err(|err| {
      err_builder.format(format_args!("failed to write to stdout: {err}"))
    })?;

  Ok(())
}
