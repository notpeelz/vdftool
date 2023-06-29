use std::io::Write;

use eyre::bail;

use crate::cli::{ArgMatchesExt, CommandExt};

pub(crate) const NAME: &str = "json2vdf";

#[cfg(not(windows))]
pub(crate) const BIN_NAME: &str = "json2vdf";

#[cfg(windows)]
pub(crate) const BIN_NAME: &str = "json2vdf.exe";

pub(crate) fn command(entrypoint: bool) -> clap::Command {
  clap::Command::new(NAME)
    .about(if entrypoint {
      "Convert JSON to VDF (Valve Data Format, aka KeyValues)"
    } else {
      "Convert JSON to VDF"
    })
    .with_file_arg()
    .arg(
      clap::Arg::new("top-level")
        .short('t')
        .long("top-level")
        .help(
          "Specify the top-level key instead of inferring it from the input",
        )
        .value_name("key")
        .num_args(1),
    )
}

pub(crate) fn main(args: &clap::ArgMatches) -> eyre::Result<()> {
  let (stream, err_builder) = args.get_input_file()?;

  let data = std::io::read_to_string(stream)
    .map_err(|err| err_builder.format(format_args!("failed to read: {err}")))?;

  if !data.as_bytes().iter().any(|b| {
    char::from_u32(*b as u32).is_some_and(|c| !['\r', '\n'].contains(&c))
  }) {
    bail!(err_builder.format(format_args!("no data")));
  }

  let json =
    serde_json::from_str::<serde_json::Value>(&data).map_err(|err| {
      err_builder.format(format_args!("failed to deserialize JSON: {err}"))
    })?;

  let key = args.get_one::<String>("top-level").map(|x| &**x);
  let (json, key) = match key {
    Some(key) => Ok((json, key.to_string())),
    None => 'a: {
      let serde_json::Value::Object(mut json) = json else {
        break 'a Err(err_builder.format(format_args!(
          "root element isn't a JSON object"
        )));
      };

      const ERR_REQUIRE_TOP_LEVEL_KEY: &str =
        "root object requires exactly one top-level key";

      let mut keys = json.keys();
      let key = if let Some(key) = keys.next() {
        key.to_string()
      } else {
        break 'a Err(
          err_builder.format(format_args!("{ERR_REQUIRE_TOP_LEVEL_KEY}")),
        );
      };

      if keys.next().is_some() {
        break 'a Err(
          err_builder.format(format_args!("{ERR_REQUIRE_TOP_LEVEL_KEY}")),
        );
      }

      let json = json.remove(&key).unwrap();
      Ok((json, key))
    },
  }?;

  let vdf =
    keyvalues_serde::to_string_with_key(&json, &key).map_err(|err| {
      err_builder.format(format_args!("failed to serialize to VDF: {err}"))
    })?;

  std::io::stdout()
    .write_all(vdf.as_bytes())
    .map_err(|err| {
      err_builder.format(format_args!("failed to write to stdout: {err}"))
    })?;

  Ok(())
}
