use std::io::Read;

use eyre::{eyre, Report, Result};

#[derive(Clone, Copy)]
pub(crate) struct ErrorBuilder<'a> {
  file: Option<&'a str>,
}

impl ErrorBuilder<'_> {
  pub fn format(&self, message: std::fmt::Arguments) -> Report {
    if let Some(file) = self.file {
      eyre!(format!("{file}: {}", message))
    } else {
      eyre!(format!("{}", message))
    }
  }
}

pub(crate) trait CommandExt {
  fn with_file_arg(self) -> Self;
}

impl CommandExt for clap::Command {
  fn with_file_arg(self) -> Self {
    self.arg(
      clap::Arg::new("file")
        .hide_long_help(true)
        .hide_short_help(true),
    )
  }
}

pub(crate) trait ArgMatchesExt {
  fn get_input_file(&self) -> Result<(Box<dyn Read>, ErrorBuilder)>;
}

impl ArgMatchesExt for clap::ArgMatches {
  fn get_input_file(&self) -> Result<(Box<dyn Read>, ErrorBuilder)> {
    let file = self.get_one::<String>("file").map(|x| &**x);
    let (stream, err_builder) = match file {
      Some("-") | None => {
        let err_builder = ErrorBuilder { file: None };

        let stream = Ok(Box::new(std::io::stdin()) as Box<dyn Read>);

        (stream, err_builder)
      },
      Some(s) => {
        let err_builder = ErrorBuilder { file: None };

        let stream = std::fs::File::open(s)
          .map(|f| Box::new(f) as Box<dyn Read>)
          .map_err(|err| err_builder.format(format_args!("{err}")));

        (stream, err_builder)
      },
    };

    stream.map(|stream| (stream, err_builder))
  }
}
