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

//! Autogenerated weights for `pallet_election_provider_multi_phase`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2023-06-18, STEPS: `50`, REPEAT: `20`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `runner-e8ezs4ez-project-163-concurrent-0`, CPU: `Intel(R) Xeon(R) CPU @ 2.60GHz`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("saitachain-dev"), DB CACHE: 1024

// Executed Command:
// ./target/production/saitachain
// benchmark
// pallet
// --chain=saitachain-dev
// --steps=50
// --repeat=20
// --no-storage-info
// --no-median-slopes
// --no-min-squares
// --pallet=pallet_election_provider_multi_phase
// --extrinsic=*
// --execution=wasm
// --wasm-execution=compiled
// --header=./file_header.txt
// --output=./runtime/saitachain/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::Weight};
use core::marker::PhantomData;

/// Weight functions for `pallet_election_provider_multi_phase`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_election_provider_multi_phase::WeightInfo for WeightInfo<T> {
	/// Storage: Staking CurrentEra (r:1 w:0)
	/// Proof: Staking CurrentEra (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Staking CurrentPlannedSession (r:1 w:0)
	/// Proof: Staking CurrentPlannedSession (max_values: Some(1), max_size: Some(4), added: 499, mode: MaxEncodedLen)
	/// Storage: Staking ErasStartSessionIndex (r:1 w:0)
	/// Proof: Staking ErasStartSessionIndex (max_values: None, max_size: Some(16), added: 2491, mode: MaxEncodedLen)
	/// Storage: Babe EpochIndex (r:1 w:0)
	/// Proof: Babe EpochIndex (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Babe GenesisSlot (r:1 w:0)
	/// Proof: Babe GenesisSlot (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Babe CurrentSlot (r:1 w:0)
	/// Proof: Babe CurrentSlot (max_values: Some(1), max_size: Some(8), added: 503, mode: MaxEncodedLen)
	/// Storage: Staking ForceEra (r:1 w:0)
	/// Proof: Staking ForceEra (max_values: Some(1), max_size: Some(1), added: 496, mode: MaxEncodedLen)
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize_nothing() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `993`
		//  Estimated: `3481`
		// Minimum execution time: 19_675_000 picoseconds.
		Weight::from_parts(20_310_000, 0)
			.saturating_add(Weight::from_parts(0, 3481))
			.saturating_add(T::DbWeight::get().reads(8))
	}
	/// Storage: ElectionProviderMultiPhase Round (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Round (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize_open_signed() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `114`
		//  Estimated: `1599`
		// Minimum execution time: 12_119_000 picoseconds.
		Weight::from_parts(12_730_000, 0)
			.saturating_add(Weight::from_parts(0, 1599))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ElectionProviderMultiPhase Round (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Round (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	fn on_initialize_open_unsigned() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `114`
		//  Estimated: `1599`
		// Minimum execution time: 13_456_000 picoseconds.
		Weight::from_parts(13_787_000, 0)
			.saturating_add(Weight::from_parts(0, 1599))
			.saturating_add(T::DbWeight::get().reads(2))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	/// Storage: ElectionProviderMultiPhase QueuedSolution (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase QueuedSolution (max_values: Some(1), max_size: None, mode: Measured)
	fn finalize_signed_phase_accept_solution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `174`
		//  Estimated: `3593`
		// Minimum execution time: 33_871_000 picoseconds.
		Weight::from_parts(34_289_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(2))
	}
	/// Storage: System Account (r:1 w:1)
	/// Proof: System Account (max_values: None, max_size: Some(128), added: 2603, mode: MaxEncodedLen)
	fn finalize_signed_phase_reject_solution() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `174`
		//  Estimated: `3593`
		// Minimum execution time: 22_897_000 picoseconds.
		Weight::from_parts(23_307_000, 0)
			.saturating_add(Weight::from_parts(0, 3593))
			.saturating_add(T::DbWeight::get().reads(1))
			.saturating_add(T::DbWeight::get().writes(1))
	}
	/// Storage: ElectionProviderMultiPhase SnapshotMetadata (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SnapshotMetadata (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase DesiredTargets (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase DesiredTargets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Snapshot (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase Snapshot (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	fn create_snapshot_internal(v: u32, _t: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `0`
		//  Estimated: `0`
		// Minimum execution time: 556_279_000 picoseconds.
		Weight::from_parts(581_580_000, 0)
			.saturating_add(Weight::from_parts(0, 0))
			// Standard Error: 3_088
			.saturating_add(Weight::from_parts(312_241, 0).saturating_mul(v.into()))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: ElectionProviderMultiPhase SignedSubmissionIndices (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionIndices (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SignedSubmissionNextIndex (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionNextIndex (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SnapshotMetadata (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SnapshotMetadata (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SignedSubmissionsMap (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionsMap (max_values: None, max_size: None, mode: Measured)
	/// Storage: System BlockWeight (r:1 w:1)
	/// Proof: System BlockWeight (max_values: Some(1), max_size: Some(48), added: 543, mode: MaxEncodedLen)
	/// Storage: ElectionProviderMultiPhase QueuedSolution (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase QueuedSolution (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Round (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase Round (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase DesiredTargets (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase DesiredTargets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Snapshot (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase Snapshot (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn elect_queued(a: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `338 + a * (768 ±0) + d * (48 ±0)`
		//  Estimated: `3890 + a * (768 ±0) + d * (49 ±0)`
		// Minimum execution time: 420_334_000 picoseconds.
		Weight::from_parts(18_023_312, 0)
			.saturating_add(Weight::from_parts(0, 3890))
			// Standard Error: 7_565
			.saturating_add(Weight::from_parts(659_974, 0).saturating_mul(a.into()))
			// Standard Error: 11_339
			.saturating_add(Weight::from_parts(287_336, 0).saturating_mul(d.into()))
			.saturating_add(T::DbWeight::get().reads(8))
			.saturating_add(T::DbWeight::get().writes(9))
			.saturating_add(Weight::from_parts(0, 768).saturating_mul(a.into()))
			.saturating_add(Weight::from_parts(0, 49).saturating_mul(d.into()))
	}
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SnapshotMetadata (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase SnapshotMetadata (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	/// Proof: TransactionPayment NextFeeMultiplier (max_values: Some(1), max_size: Some(16), added: 511, mode: MaxEncodedLen)
	/// Storage: ElectionProviderMultiPhase SignedSubmissionIndices (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionIndices (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SignedSubmissionNextIndex (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionNextIndex (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SignedSubmissionsMap (r:0 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase SignedSubmissionsMap (max_values: None, max_size: None, mode: Measured)
	fn submit() -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `1204`
		//  Estimated: `2689`
		// Minimum execution time: 49_669_000 picoseconds.
		Weight::from_parts(52_076_000, 0)
			.saturating_add(Weight::from_parts(0, 2689))
			.saturating_add(T::DbWeight::get().reads(5))
			.saturating_add(T::DbWeight::get().writes(3))
	}
	/// Storage: ElectionProviderMultiPhase CurrentPhase (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase CurrentPhase (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Round (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Round (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase DesiredTargets (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase DesiredTargets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase QueuedSolution (r:1 w:1)
	/// Proof Skipped: ElectionProviderMultiPhase QueuedSolution (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase SnapshotMetadata (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase SnapshotMetadata (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Snapshot (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Snapshot (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase MinimumUntrustedScore (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase MinimumUntrustedScore (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn submit_unsigned(v: u32, t: u32, a: u32, _d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `219 + t * (32 ±0) + v * (553 ±0)`
		//  Estimated: `1704 + t * (32 ±0) + v * (553 ±0)`
		// Minimum execution time: 5_966_688_000 picoseconds.
		Weight::from_parts(6_129_265_000, 0)
			.saturating_add(Weight::from_parts(0, 1704))
			// Standard Error: 20_174
			.saturating_add(Weight::from_parts(154_243, 0).saturating_mul(v.into()))
			// Standard Error: 59_786
			.saturating_add(Weight::from_parts(5_709_666, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(7))
			.saturating_add(T::DbWeight::get().writes(1))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(t.into()))
			.saturating_add(Weight::from_parts(0, 553).saturating_mul(v.into()))
	}
	/// Storage: ElectionProviderMultiPhase DesiredTargets (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase DesiredTargets (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Snapshot (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Snapshot (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase Round (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase Round (max_values: Some(1), max_size: None, mode: Measured)
	/// Storage: ElectionProviderMultiPhase MinimumUntrustedScore (r:1 w:0)
	/// Proof Skipped: ElectionProviderMultiPhase MinimumUntrustedScore (max_values: Some(1), max_size: None, mode: Measured)
	/// The range of component `v` is `[1000, 2000]`.
	/// The range of component `t` is `[500, 1000]`.
	/// The range of component `a` is `[500, 800]`.
	/// The range of component `d` is `[200, 400]`.
	fn feasibility_check(v: u32, t: u32, a: u32, _d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `194 + t * (32 ±0) + v * (553 ±0)`
		//  Estimated: `1679 + t * (32 ±0) + v * (553 ±0)`
		// Minimum execution time: 5_058_457_000 picoseconds.
		Weight::from_parts(5_216_393_000, 0)
			.saturating_add(Weight::from_parts(0, 1679))
			// Standard Error: 15_829
			.saturating_add(Weight::from_parts(278_945, 0).saturating_mul(v.into()))
			// Standard Error: 46_908
			.saturating_add(Weight::from_parts(3_239_889, 0).saturating_mul(a.into()))
			.saturating_add(T::DbWeight::get().reads(4))
			.saturating_add(Weight::from_parts(0, 32).saturating_mul(t.into()))
			.saturating_add(Weight::from_parts(0, 553).saturating_mul(v.into()))
	}
}
