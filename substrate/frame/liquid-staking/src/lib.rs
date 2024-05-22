#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::DispatchResult,
	PalletId,
	traits::{
		fungible::{ Inspect, Mutate },
		fungibles::{ Inspect as Inspects, Mutate as Mutates },
		liquid_staking::{ StakingAccount, DerivativeRewardAccount },
		LockableCurrency,
		tokens::{ Precision::BestEffort, Preservation::Expendable, Fortitude::Polite },
	},
};
use liquid_staking_primitives::{ EraIndex, CurrencyId, Balance };
use scale_info::prelude::vec::Vec;
use pallet_staking::{ CurrentEra, UnlockChunk };
use crate::types::{ MatchingLedger, LiquidStakingCurrenciesProvider, DecimalProvider };
use sp_runtime::traits::{ StaticLookup, Zero, AccountIdConversion };
use pallet_reward::NominatorRewardAccounts;
pub type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
pub type BalanceOf<T> =
	<<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;
pub type AssetIdOf<T> =
	<<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;

pub use pallet::*;
pub mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::{ ValueQuery, * };
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_staking::Config + pallet_reward::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Assets: Mutates<Self::AccountId, AssetId = CurrencyId, Balance = Balance> +
			Inspects<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;
		type LiquidCurrency: Get<AssetIdOf<Self>>;
		type Decimal: DecimalProvider<CurrencyId>;
		type Balances: Inspect<Self::AccountId, Balance = Balance> +
			Mutate<Self::AccountId, Balance = Balance> +
			LockableCurrency<Self::AccountId, Balance = Balance, Moment = BlockNumberFor<Self>>;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		#[pallet::constant]
		type MinStake: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn unlockings)]
	pub type Unlockings<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		Vec<UnlockChunk<BalanceOf<T>>>,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn account_list)]
	pub type AccountStake<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u128,
		OptionQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn matching_pool)]
	pub type MatchingPool<T: Config> = StorageValue<_, MatchingLedger<BalanceOf<T>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn bonded_accounts)]
	pub type StakedAccounts<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn era_reward_account)]
	pub type EraDerivativeReward<T: Config> = StorageValue<_, Vec<T::AccountId>, OptionQuery>;

    #[pallet::storage]
    pub type Bonds<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The assets get staked successfully
		Staked(T::AccountId, BalanceOf<T>),
		/// The assets got unstaked successfully
		UnStaked(T::AccountId, BalanceOf<T>),
		/// The amount will be unlocked at target era
		Unlocked(BalanceOf<T>, EraIndex),
		/// Claim user's unbonded staking assets
		ClaimedFor(T::AccountId, BalanceOf<T>),
		/// the reward will be distributed after era
		Rewarded(T::AccountId),
		/// the reward has been successfully received
		DerivativeReceived(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Invalid liquid currency
		InvalidLiquidCurrency,
		/// Stash wasn't bonded yet
		NotStaked,
		/// The stake was below the minimum, `MinStake`.
		StakeTooSmall,
		/// There is no nothing to claim
		NothingToClaim,
		/// There is no unlocking available for the account
		NoUnlockings,
		/// Please wait for the era to complete
		WaitTheEraToComplete,
		/// Not enough amount have been staked
		InsufficientBalance,
		/// Not enough amount have been bonded
		InsufficientBonded,
		/// No amount have been bonded
		NotBonded,
		/// No account in era reward accounts
		AccountNotInDerivativeReward,
		/// Not enough stake to nominate
    	CannotNominate,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn stake(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			ensure!(amount >= T::MinStake::get(), Error::<T>::StakeTooSmall);
			let liquid_currency = Self::liquid_currency()?;
			T::Assets::mint_into(liquid_currency, &who, amount)?;
			Self::bonded_account(who.clone())?;
			T::Balances::transfer(&who, &Self::account_id(), amount, Expendable)?;
			Self::update_share(who.clone(), amount)?;
			MatchingPool::<T>::try_mutate(|p| -> DispatchResult { p.add_stake_amount(amount) })?;
			Self::deposit_event(Event::<T>::Staked(who, amount.into()));
			Ok(().into())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn unstake(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let staked_accounts = StakedAccounts::<T>::get();
			ensure!(staked_accounts.contains(&who), Error::<T>::NotStaked);
			T::Assets::burn_from(Self::liquid_currency()?, &who, amount, BestEffort, Polite)?;
			let staked_amount = AccountStake::<T>::get(who.clone()).unwrap_or(0);
			ensure!(amount <= staked_amount, Error::<T>::InsufficientBalance);
			let new_amount = staked_amount - amount;
			if new_amount.is_zero() {
				let mut account = StakedAccounts::<T>::get();
				if let Some(index) = account.iter().position(|x| *x == who) {
					account.remove(index);
					StakedAccounts::<T>::put(&account);
				}
				AccountStake::<T>::remove(who.clone());
			} else {
				AccountStake::<T>::insert(who.clone(), new_amount);
			}
			Unlockings::<T>::try_mutate(&who, |b| -> DispatchResult {
				let mut chunks = b.take().unwrap_or_default();
				let target_era = Self::target_era();
				if let Some(chunk) = chunks.last_mut().filter(|chunk| chunk.era == target_era) {
					chunk.value = chunk.value.saturating_add(amount);
				} else {
					chunks.push(UnlockChunk {
						value: amount,
						era: target_era,
					});
				}
				*b = Some(chunks);
				Self::deposit_event(Event::<T>::Unlocked(amount, target_era));
				Ok(())
			})?;
			MatchingPool::<T>::try_mutate(|p| p.add_unstake_amount(amount))?;
			Self::deposit_event(Event::<T>::UnStaked(who, amount));
			Ok(().into())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn claim_for(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			let who = T::Lookup::lookup(dest)?;
			let current_era = CurrentEra::<T>::get().unwrap_or(0);

			Unlockings::<T>::try_mutate_exists(&who, |b| -> DispatchResult {
				let mut amount: BalanceOf<T> = Zero::zero();
				let chunks = b.as_mut().ok_or(Error::<T>::NoUnlockings)?;
				chunks.retain(|chunk| {
					if chunk.era > current_era {
						true
					} else {
						amount += chunk.value;
						false
					}
				});
				if amount.is_zero() {
					return Err(Error::<T>::NothingToClaim.into());
				}
				Self::do_claim_for(&who, amount)?;

				if chunks.is_empty() {
					*b = None;
				}

				Self::deposit_event(Event::<T>::ClaimedFor(who.clone(), amount));
				Ok(())
			})?;
			Ok(().into())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn bond(origin: OriginFor<T>, balance: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			<Bonds<T>>::insert(&sender, balance);
			pallet_staking::Pallet::<T>::bond(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id().clone())),
				balance.into(),
				pallet_staking::RewardDestination::Account(Self::account_id())
			)?;
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn unbond(origin: OriginFor<T>, balance: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			let mut bonded_amount = <Bonds<T>>::get(&sender).ok_or(Error::<T>::NotBonded)?;
            ensure!(bonded_amount >= balance, Error::<T>::InsufficientBonded);
			pallet_staking::Pallet::<T>::unbond(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id().clone())),
				balance.into()
			);
			bonded_amount -= balance;
            if bonded_amount.is_zero() {
            // If the entire bond is unbonded, remove the record from storage
            <Bonds<T>>::remove(&sender);
           } else {
           // Update the bond amount in storage
           <Bonds<T>>::insert(&sender, bonded_amount);
           }
           Ok(())
		}

		
		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn withdraw_unbonded(_origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			pallet_staking::Pallet::<T>::withdraw_unbonded(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id())),
				0
			)
		}

		#[pallet::call_index(7)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn nominate(
			origin: OriginFor<T>,
			targets: Vec<AccountIdLookupOf<T>>
		) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;
			let staked_accounts = StakedAccounts::<T>::get();
			ensure!(staked_accounts.contains(&who), Error::<T>::NotStaked);
			let staked_amount = AccountStake::<T>::get(who.clone()).unwrap_or(0);
			// Access the `AccountStake` storage
			let account_stake_storage = <AccountStake<T>>::iter();
			// Iterate through all entries and sum up the stake amounts
			let mut total_staked_amount = 0;
			for (_, stake) in account_stake_storage {
				if let stake_amount = stake {
					total_staked_amount += stake_amount;
				}
			}
			let min_stake = pallet_staking::MinNominatorBond::<T>::get();
			ensure!(total_staked_amount >= min_stake.into() , Error::<T>:: CannotNominate);
			pallet_staking::Pallet::<T>::nominate(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id())),
				targets
			)?;
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn claim_reward(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let staked_accounts = StakedAccounts::<T>::get();
			ensure!(staked_accounts.contains(&who), Error::<T>::NotStaked);
			let mut era_reward_accounts = EraDerivativeReward::<T>::get().unwrap_or_else(Vec::new);
			if era_reward_accounts.is_empty() {
				era_reward_accounts.push(who.clone());
				EraDerivativeReward::<T>::put(era_reward_accounts.clone());
				if let Some(nominator) = pallet_staking::Nominators::<T>::get(Self::account_id()) {
					for target in &nominator.targets {
						pallet_reward::Pallet::<T>::get_rewards(
							T::RuntimeOrigin::from(
								frame_system::RawOrigin::Signed(Self::account_id())
							),
							target.clone()
						)?;
					}
				} else {
					return Ok(());
				}
			} else {
				ensure!(!era_reward_accounts.contains(&who), Error::<T>::WaitTheEraToComplete);
				era_reward_accounts.push(who.clone());
				EraDerivativeReward::<T>::put(era_reward_accounts.clone());
			}
			Self::deposit_event(Event::<T>::Rewarded(who));

			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn bond_extra(_origin: OriginFor<T>, extra_balance: BalanceOf<T>) -> DispatchResult {
			pallet_staking::Pallet::<T>::bond_extra(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id().clone())),
				extra_balance.into()
			)?;
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(Weight::from_parts(10_000, 0) + T::DbWeight::get().writes(1))]
		pub fn rebond(origin: OriginFor<T>, balance: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let staked_accounts = StakedAccounts::<T>::get();
			ensure!(staked_accounts.contains(&who), Error::<T>::NotStaked);
			pallet_staking::Pallet::<T>::rebond(
				T::RuntimeOrigin::from(frame_system::RawOrigin::Signed(Self::account_id().clone())),
				balance.into()
			)
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
		pub fn liquid_currency() -> Result<AssetIdOf<T>, DispatchError> {
			Self::get_liquid_currency().ok_or(Error::<T>::InvalidLiquidCurrency).map_err(Into::into)
		}
		pub fn target_era() -> EraIndex {
			pallet_staking::Pallet::<T>::current_era().unwrap_or(0) + T::BondingDuration::get() + 1
		}
		pub fn update_share(account: T::AccountId, amount: u128) -> DispatchResult {
			let already_present_amount = AccountStake::<T>::get(account.clone()).unwrap_or(0);
			let new_amount = already_present_amount + amount;
			AccountStake::<T>::insert(account.clone(), new_amount);
			Ok(())
		}
		pub fn bonded_account(who: T::AccountId) -> DispatchResult {
			let mut bonded_accounts = StakedAccounts::<T>::get();
			if bonded_accounts.contains(&who) {
				return Ok(());
			}
			bonded_accounts.push(who.clone());
			StakedAccounts::<T>::put(bonded_accounts);
			Ok(())
		}
		fn do_claim_for(who: &T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			T::Balances::transfer(&Self::account_id(), who, amount, Expendable)?;
			Ok(())
		}

		pub fn convert_f64_to_u128(value: f64) -> u128 {
			let precision = 18;
			let multiplier = (10u128).pow(precision);
			let number = (value * (multiplier as f64)) as u128;
			number
		}

		pub fn calculate_reward(share: u128, total_stake: u128, reward: u128) -> f64 {
			let precision = 18;
			let scaled_share = share / (10u128).pow(precision);
			let scaled_total_stake: u64 = (total_stake / (10u128).pow(precision)) as u64;
			let division: f64 = ((scaled_share as f64) / (scaled_total_stake as f64)) as f64;
			let scaled_reward: f64 = ((reward as f64) / ((10u128).pow(precision) as f64)) as f64;
			let total_reward = division * scaled_reward;
			total_reward
		}
	}
}

impl<T: Config> DerivativeRewardAccount<T::AccountId> for Pallet<T> {
	fn derivative_reward_accounts() -> Vec<T::AccountId> {
		EraDerivativeReward::<T>::get().unwrap_or_else(Vec::new)
	}

	fn claim_derivative(account: T::AccountId) -> DispatchResult {
		let individual_stake = AccountStake::<T>::get(account.clone()).unwrap_or(0);
		let pool = MatchingPool::<T>::get();
		let total_stake = pool.total_stake_amount.total;
		let nominator_reward = NominatorRewardAccounts::<T>::get(Self::account_id());
		let individual_reward = Self::calculate_reward(
			individual_stake,
			total_stake,
			nominator_reward.into()
		);
		let liquid_currency = Self::liquid_currency()?;
		let mut era_reward_accounts = EraDerivativeReward::<T>::get().unwrap_or_else(Vec::new);
		if !era_reward_accounts.contains(&account) {
			return Err(Error::<T>::AccountNotInDerivativeReward.into());
		}
		if let Some(index) = era_reward_accounts.iter().position(|a| a == &account.clone()) {
			era_reward_accounts.remove(index);
		}
		let converted_reward = Self::convert_f64_to_u128(individual_reward);
		EraDerivativeReward::<T>::put(era_reward_accounts);
		T::Assets::mint_into(liquid_currency, &account, converted_reward)?;
		Self::update_share(account.clone(), converted_reward)?;
		Self::deposit_event(Event::<T>::DerivativeReceived(account));
		Ok(())
	}
	fn reset_reward() -> DispatchResult {
		NominatorRewardAccounts::<T>::remove(Self::account_id());
		Ok(())
	}
}

impl<T: Config> StakingAccount<T::AccountId> for Pallet<T> {
	fn staking_account() -> T::AccountId {
		Self::account_id()
	}
}
