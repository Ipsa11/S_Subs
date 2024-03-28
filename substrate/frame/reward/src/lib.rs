// Reward Pallet
// The Reward Pallet is a module in the Substrate blockchain framework designed to manage and distribute rewards to participants based on their contributions within the network.
// This pallet facilitates the allocation of rewards to validators and nominators for their involvement in staking activities.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use pallet_staking::{ CurrentEra, Validators, ErasRewardPoints, ErasStakers, IndividualExposure };
use pallet_treasury::TreasuryAccountId;
use parity_scale_codec::Codec;
use frame_support::traits::liquid_staking::StakingAccount;
use scale_info::prelude::{ vec::Vec, fmt::Debug };
use sp_runtime::{ FixedPointOperand, traits::{ Convert, AtLeast32BitUnsigned } };
use frame_support::traits::{
	Currency,
	LockableCurrency,
	ValidatorSet,
	Get,
	reward::Rewards,
	ExistenceRequirement,
	ExistenceRequirement::KeepAlive,
};
use frame_election_provider_support::{ ElectionDataProvider, DataProviderBounds };

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_staking::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type ValidatorSet: frame_support::traits::ValidatorSetWithIdentification<Self::AccountId>;
		type Validators: frame_support::traits::ValidatorSet<Self::AccountId>;
		type DataProvider: ElectionDataProvider<
			AccountId = <Self::ValidatorSet as ValidatorSet<Self::AccountId>>::ValidatorId,
			BlockNumber = BlockNumberFor<Self>
		>;
		type LiquidStakeVault: StakingAccount<Self::AccountId>;
		type ValidatorId: Convert<
			<<Self as Config>::Validators as ValidatorSet<<Self as frame_system::Config>::AccountId>>::ValidatorId,
			Option<Self::AccountId>
		>;
		type ValidatorIdOf: Convert<
			Self::AccountId,
			Option<<<Self as Config>::ValidatorSet as ValidatorSet<<Self as frame_system::Config>::AccountId>>::ValidatorId>
		>;
		type Balance: Parameter +
			Member +
			AtLeast32BitUnsigned +
			Codec +
			Default +
			From<u128> +
			Into<u128> +
			Copy +
			MaybeSerializeDeserialize +
			Debug +
			MaxEncodedLen +
			TypeInfo +
			FixedPointOperand;

		#[pallet::constant]
		type TotalReward: Get<u32>;

		type Precision: Get<u32>;

		#[pallet::constant]
		type TotalMinutesPerYear: Get<u32>;
		type EraMinutes: Get<u32>;
		type TreasuryAccount: TreasuryAccountId<Self::AccountId>;
		type RewardCurrency: LockableCurrency<
			Self::AccountId,
			Moment = BlockNumberFor<Self>,
			Balance = Self::Balance
		>;
	}

	/// The era reward which are distributed among the validator abd nominator
	#[pallet::storage]
	#[pallet::getter(fn total_rewards)]
	pub type BeneficialRewardRecord<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery
	>;

	/// Specifics regarding the rewards distributed within the designated era
	#[pallet::storage]
	#[pallet::getter(fn validator_reward_accounts)]
	pub type ValidatorRewardAccounts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery
	>;

	#[pallet::storage]
	#[pallet::getter(fn nominator_reward_accounts)]
	pub type NominatorRewardAccounts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		T::Balance,
		ValueQuery
	>;

	/// Era reward accounts
	#[pallet::storage]
	#[pallet::getter(fn era_reward)]
	pub type EraRewardsVault<T: Config> = StorageValue<_, Vec<T::AccountId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The rewards has been succesfully distributed
		Distributed {
			who: T::AccountId,
			balance: T::Balance,
		},
		/// The reward will be distributed after completely the era
		Rewarded {
			who: T::AccountId,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No validator is present
		NoSuchValidator,
		/// No Reward is available for the author
		NoRewardAvailable,
		/// No Reward is available for the author
		AlreadyRewarded,
		/// No Such Nominator is present
		NoSuchNominator,
		/// Wait for the era to complete
		WaitTheEraToComplete,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn get_rewards(origin: OriginFor<T>, validator: T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let mut era_reward_accounts = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
			ensure!(!era_reward_accounts.contains(&validator), Error::<T>::WaitTheEraToComplete);
			era_reward_accounts.push(validator.clone());
			EraRewardsVault::<T>::put(era_reward_accounts);
			Self::deposit_event(Event::<T>::Rewarded { who: validator });
			Ok(())
		}
	}
}

impl<T: Config> Rewards<T::AccountId> for Pallet<T> {
	fn payout_validators() -> Vec<T::AccountId> {
		let validators = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		validators
	}

	fn claim_rewards(account: T::AccountId) -> DispatchResult {
		let nominators = Self::check_nominators(account.clone());
		if nominators.is_empty() {
			Self::distribute_validator_reward(account.clone())?;
			Self::recalculate_rewarded_accounts(account.clone())?;
			return Ok(());
		}
		nominators.iter().for_each(|nominator| {
			let _ = Self::distribute_validator_reward(account.clone());
			let _ = Self::distribute_nominator_reward(nominator.who.clone());
		});
		Self::recalculate_rewarded_accounts(account.clone())?;
		return Ok(());
	}

	fn calculate_reward() -> DispatchResult {
		let validators = T::Validators::validators();

		validators.iter().for_each(|validator_id| {
			let validator = T::ValidatorId::convert(validator_id.clone()).unwrap();
			let validator_points = Self::get_validator_point(validator.clone());
			let era_reward = Self::calculate_era_reward();
			let total_reward = (era_reward as f64) * (validators.len() as f64);
			let reward = Self::calculate_validator_reward(validator_points.into(), total_reward);
			let nominators = Self::check_nominators(validator.clone());
			if nominators.is_empty() {
				Self::add_validator_reward(
					validator.clone(),
					Self::convert_f64_to_u128(reward).into()
				);
				return;
			}

			let validator_prefs = Validators::<T>::get(validator.clone());
			let validator_commission = validator_prefs.commission.deconstruct();
			let precision: u32 = 7;
			let scaled_commission: u32 = validator_commission / (10u32).pow(precision);
			let nominator_share = ((reward as f64) * (scaled_commission as f64)) / 100.0;
			let validator_share = reward - nominator_share;
			Self::add_validator_reward(
				validator.clone(),
				Self::convert_f64_to_u128(validator_share).into()
			);

			nominators.iter().for_each(|nominator| {
				let nominator_stake = nominator.value;
				let exposure = ErasStakers::<T>::get(Self::current_era(), validator.clone());
				let total_stake = exposure.total;
				let nominator_reward = Self::calculate_nominator_reward(
					nominator_stake.into(),
					total_stake.into(),
					nominator_share.into()
				);
				Self::add_nominator_reward(
					nominator.who.clone(),
					Self::convert_f64_to_u128(nominator_reward).into()
				);
			});
		});
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	fn transfer(
		who: T::AccountId,
		dest: T::AccountId,
		amount: T::Balance,
		existence_requirement: ExistenceRequirement
	) -> DispatchResult {
		T::RewardCurrency::transfer(&who, &dest, amount, existence_requirement)?;
		Self::deposit_event(Event::Distributed { who: dest, balance: amount });
		Ok(())
	}

	fn treasury_account() -> T::AccountId {
		T::TreasuryAccount::accountid()
	}

	fn verify_validator(who: T::AccountId) -> DispatchResult {
		let all_validators = T::DataProvider::electable_targets(
			DataProviderBounds::default()
		).unwrap();
		let val = all_validators
			.iter()
			.any(|c| T::ValidatorIdOf::convert(who.clone()) == Some(c.clone()));
		if val {
			return Ok(());
		} else {
			return Err(Error::<T>::NoSuchValidator.into());
		}
	}

	fn recalculate_rewarded_accounts(account: T::AccountId) -> DispatchResult {
		let mut era_reward_accounts = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		if let Some(index) = era_reward_accounts.iter().position(|a| a == &account.clone()) {
			era_reward_accounts.remove(index);
		}
		EraRewardsVault::<T>::put(era_reward_accounts);
		Ok(())
	}
	pub fn calculate_era_reward() -> f64 {
		let total_minutes_per_year = T::TotalMinutesPerYear::get();
		let era_minutes = T::EraMinutes::get();
		let era = total_minutes_per_year / era_minutes;
		let total_reward = T::TotalReward::get();
		let era_reward = total_reward / era;
		era_reward.into()
	}

	pub fn calculate_nominator_reward(share: u128, total_stake: u128, reward: f64) -> f64 {
		let precision = T::Precision::get();
		let scaled_share = share / (10u128).pow(precision);
		let scaled_total_stake: u64 = (total_stake / (10u128).pow(precision)) as u64;
		let division: f64 = ((scaled_share as f64) / (scaled_total_stake as f64)) as f64;
		let total_reward = division * reward;
		total_reward
	}
	pub fn convert_f64_to_u128(value: f64) -> u128 {
		let precision = T::Precision::get();
		let multiplier = (10u128).pow(precision);
		let number = (value * (multiplier as f64)) as u128;
		number
	}

	fn add_validator_reward(account: T::AccountId, reward: T::Balance) {
		let earlier_reward = ValidatorRewardAccounts::<T>::get(account.clone());
		let new_individual_reward = reward + earlier_reward;
		ValidatorRewardAccounts::<T>::insert(account.clone(), new_individual_reward);
	}

	fn add_nominator_reward(account: T::AccountId, reward: T::Balance) {
		let earlier_reward = NominatorRewardAccounts::<T>::get(account.clone());
		let new_individual_reward = reward + earlier_reward;
		NominatorRewardAccounts::<T>::insert(account.clone(), new_individual_reward);
	}

	fn get_validator_point(account: T::AccountId) -> u32 {
		let era_reward_points = <ErasRewardPoints<T>>::get(Self::active_era());
		let validator_points = era_reward_points.individual.get(&account).unwrap_or(&0);
		*validator_points
	}

	fn distribute_validator_reward(account: T::AccountId) -> DispatchResult {
		let reward = ValidatorRewardAccounts::<T>::get(account.clone());
		Self::transfer(Self::treasury_account(), account.clone(), reward, KeepAlive)?;
		ValidatorRewardAccounts::<T>::remove(account.clone());
		BeneficialRewardRecord::<T>::insert(account.clone(), reward);
		Ok(())
	}

	fn distribute_nominator_reward(account: T::AccountId) -> DispatchResult {
		let reward = NominatorRewardAccounts::<T>::get(account.clone());
		T::RewardCurrency::transfer(&Self::treasury_account(), &account, reward, KeepAlive)?;
		let staking_account = T::LiquidStakeVault::staking_account();
		if account != staking_account {
			NominatorRewardAccounts::<T>::remove(account.clone());
		}
		BeneficialRewardRecord::<T>::insert(account.clone(), reward);
		Ok(())
	}

	fn current_era() -> u32 {
		CurrentEra::<T>::get().unwrap_or(0)
	}

	fn active_era() -> u32 {
		let active_era = pallet_staking::Pallet::<T>::active_era().unwrap();
		let era = active_era.index;
		era
	}

	fn calculate_validator_reward(validator_points: u32, era_reward: f64) -> f64 {
		let era_reward_points = <ErasRewardPoints<T>>::get(Self::active_era());
		let total_points = era_reward_points.total as u32;
		let reward = ((validator_points as f64) / (total_points as f64)) * (era_reward as f64);
		reward
	}

	fn check_nominators(
		who: T::AccountId
	) -> Vec<IndividualExposure<T::AccountId, <T as pallet_staking::Config>::CurrencyBalance>> {
		let exposure = ErasStakers::<T>::get(Self::current_era(), who.clone());
		let nominators = exposure.others;
		nominators
	}
}
