mod cli_opts;

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};

use anyhow::Result;
use node_cli::service::RuntimeApi;
use node_cli::service::Block;
use substrate_archive::{Archive, ArchiveBuilder, SecondaryRocksDb};

pub fn main() -> Result<()> {
	let cli = cli_opts::CliOpts::init();
	let config = cli.parse()?;

	let spec = match cli.chain_spec.to_ascii_lowercase().as_str() {
		"dev" | "" => node_cli::chain_spec::development_config(),
		"local" => node_cli::chain_spec::local_testnet_config(),
		path => node_cli::chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path)).unwrap(),
	};
	let mut archive = ArchiveBuilder::<Block, RuntimeApi, SecondaryRocksDb>::with_config(config)
		.chain_spec(Box::new(spec))
		.build()?;
	archive.drive()?;
	let running = Arc::new(AtomicBool::new(true));
	let r = running.clone();

	ctrlc::set_handler(move || {
		r.store(false, Ordering::SeqCst);
	})
	.expect("Error setting Ctrl-C handler");
	while running.load(Ordering::SeqCst) {}
	archive.shutdown()?;

	Ok(())
}
