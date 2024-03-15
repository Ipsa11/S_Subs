
//! Test to execute the snapshot using the voter bag.

use frame_election_provider_support::{
	bounds::{CountBound, DataProviderBounds},
	SortedListProvider,
};
use frame_support::traits::PalletInfoAccess;
use remote_externalities::{Builder, Mode, OnlineConfig};
use sp_runtime::{traits::Block as BlockT, DeserializeOwned};

/// Execute create a snapshot from pallet-staking.
pub async fn execute<Runtime, Block>(voter_limit: Option<usize>, currency_unit: u64, ws_url: String)
where
	Runtime: crate::RuntimeT<pallet_bags_list::Instance1>,
	Block: BlockT + DeserializeOwned,
	Block::Header: DeserializeOwned,
{
	use frame_support::storage::generator::StorageMap;

	let mut ext = Builder::<Block>::new()
		.mode(Mode::Online(OnlineConfig {
			transport: ws_url.to_string().into(),
			// NOTE: we don't scrape pallet-staking, this kinda ensures that the source of the data
			// is bags-list.
			pallets: vec![pallet_bags_list::Pallet::<Runtime, pallet_bags_list::Instance1>::name()
				.to_string()],
			at: None,
			hashed_prefixes: vec![
				<pallet_staking::Bonded<Runtime>>::prefix_hash(),
				<pallet_staking::Ledger<Runtime>>::prefix_hash(),
				<pallet_staking::Validators<Runtime>>::map_storage_final_prefix(),
				<pallet_staking::Nominators<Runtime>>::map_storage_final_prefix(),
			],
			hashed_keys: vec![
				<pallet_staking::Validators<Runtime>>::counter_storage_final_key().to_vec(),
				<pallet_staking::Nominators<Runtime>>::counter_storage_final_key().to_vec(),
			],
			..Default::default()
		}))
		.build()
		.await
		.unwrap();

	ext.execute_with(|| {
		use frame_election_provider_support::ElectionDataProvider;
		log::info!(
			target: crate::LOG_TARGET,
			"{} nodes in bags list.",
			<Runtime as pallet_staking::Config>::VoterList::count(),
		);

		let bounds = match voter_limit {
			None => DataProviderBounds::default(),
			Some(v) => DataProviderBounds { count: Some(CountBound(v as u32)), size: None },
		};

		let voters =
			<pallet_staking::Pallet<Runtime> as ElectionDataProvider>::electing_voters(bounds)
				.unwrap();

		let mut voters_nominator_only = voters
			.iter()
			.filter(|(v, _, _)| pallet_staking::Nominators::<Runtime>::contains_key(v))
			.cloned()
			.collect::<Vec<_>>();
		voters_nominator_only.sort_by_key(|(_, w, _)| *w);

		let currency_unit = currency_unit as f64;
		let min_voter = voters_nominator_only
			.first()
			.map(|(x, y, _)| (x.clone(), *y as f64 / currency_unit));
		let max_voter = voters_nominator_only
			.last()
			.map(|(x, y, _)| (x.clone(), *y as f64 / currency_unit));
		log::info!(
			target: crate::LOG_TARGET,
			"a snapshot with limit {:?} has been created, {} voters are taken. min nominator: {:?}, max: {:?}",
			voter_limit,
			voters.len(),
			min_voter,
			max_voter
		);
	});
}
