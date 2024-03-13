#![cfg(test)]
use crate::{ self as pallet_burn };
use frame_support::{ parameter_types, traits::{ ConstU16, ConstU64 } };
use sp_runtime::BuildStorage;
use sp_runtime::traits::IdentityLookup;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::testing::H256;
use frame_support::traits::ConstU128;
use frame_support::pallet_prelude::ConstU32;
use saitama_core_primitives::BlockNumber;
use frame_system::EnsureRoot;
pub type BalanceBurn = pallet_balances::Pallet<Test>;
use frame_support::pallet_prelude::Weight;

use frame_support::PalletId;

parameter_types! {
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(Weight::MAX);
}

frame_support::construct_runtime!(
	pub enum Test	
	{
        System: frame_system::{Pallet, Call, Config<T>, Storage, Event<T>},
		Burn: pallet_burn,
        Balances: pallet_balances,
	}
);

type Block = frame_system::mocking::MockBlock<Test>;

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u128>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

const MOTION_DURATION_IN_BLOCKS: BlockNumber = 3;
parameter_types! {

	pub const MotionDuration: BlockNumber = MOTION_DURATION_IN_BLOCKS;
	pub const MaxProposals: u32 = 100;
	pub const MaxMembers: u32 = 100;
    pub MaxProposalWeight: Weight = sp_runtime::Perbill::from_percent(50) * BlockWeights::get().max_block;
    
}


parameter_types! {
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const BurnPalletId: PalletId = PalletId(*b"py/burns");
}

impl pallet_balances::Config for Test {
	type Balance = u128;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<100>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type MaxHolds = ();
}

impl pallet_burn::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type BurnOrigin = EnsureRoot<Self::AccountId>;
	type PalletId = BurnPalletId;
	type Currency = Balances;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}
