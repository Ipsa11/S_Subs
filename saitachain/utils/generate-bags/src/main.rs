// Copyright (C) Saitama (UK) Ltd.
// This file is part of SaitaChain.

// Saitama is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Saitama is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with SaitaChain.  If not, see <http://www.gnu.org/licenses/>.

//! Make the set of voting bag thresholds to be used in `voter_bags.rs`.
//!
//! Generally speaking this script can be run once per runtime and never
//! touched again. It can be reused to regenerate a wholly different
//! quantity of bags, or if the existential deposit changes, etc.

use clap::{Parser, ValueEnum};
use generate_bags::generate_thresholds;
use saitama_runtime::Runtime as SaitaChainRuntime;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, ValueEnum)]
#[value(rename_all = "PascalCase")]
enum Runtime {
	SaitaChain,
}

impl Runtime {
	fn generate_thresholds_fn(
		&self,
	) -> Box<dyn FnOnce(usize, &Path, u128, u128) -> Result<(), std::io::Error>> {
		match self {
			Runtime::SaitaChain => Box::new(generate_thresholds::<SaitaChainRuntime>),
		}
	}
}

#[derive(Debug, Parser)]
struct Opt {
	/// How many bags to generate.
	#[arg(long, default_value_t = 200)]
	n_bags: usize,

	/// Which runtime to generate.
	#[arg(long, ignore_case = true, value_enum, default_value_t = Runtime::SaitaChain)]
	runtime: Runtime,

	/// Where to write the output.
	output: PathBuf,

	/// The total issuance of the native currency.
	#[arg(short, long)]
	total_issuance: u128,

	/// The minimum account balance (i.e. existential deposit) for the native currency.
	#[arg(short, long)]
	minimum_balance: u128,
}

fn main() -> Result<(), std::io::Error> {
	let Opt { n_bags, output, runtime, total_issuance, minimum_balance } = Opt::parse();

	runtime.generate_thresholds_fn()(n_bags, &output, total_issuance, minimum_balance)
}
