// Copyright 2018-2019 Parity Technologies (UK) Ltd.
// This file is part of substrate-archive.

// substrate-archive is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// substrate-archive is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with substrate-archive.  If not, see <http://www.gnu.org/licenses/>.

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use substrate_archive::ArchiveConfig;

#[derive(Clone, Debug, Parser)]
pub struct CliOpts {
    /// Sets a custom config file
    #[clap(short = 'c', long, name = "FILE", default_value = "archive.toml")]
    pub config: PathBuf,

    #[clap(short = 's', long = "chain", name = "CHAIN", default_value = "dev")]
    pub chain_spec: String,
}

impl CliOpts {
    pub fn init() -> Self {
        Parser::parse()
    }

    pub fn parse(&self) -> Result<Option<ArchiveConfig>> {
        let toml_str = fs::read_to_string(self.config.as_path())?;
        let config = toml::from_str::<ArchiveConfig>(toml_str.as_str())?;
        Ok(Some(config))
    }
}
