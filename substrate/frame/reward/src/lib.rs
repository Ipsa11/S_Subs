// Reward Pallet
// The Reward Pallet is a module in the Substrate blockchain framework designed to manage and distribute rewards to participants based on their contributions within the network.
// This pallet facilitates the allocation of rewards to validators and nominators for their involvement in staking activities.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::DispatchResult;
use frame_support::ensure;
pub use pallet::*;
use pallet_staking::{ CurrentEra, Validators, ErasRewardPoints, ErasStakers, IndividualExposure };
use pallet_treasury::TreasuryAccountId;
use sp_runtime::traits::AtLeast32BitUnsigned;
use parity_scale_codec::Codec;
use scale_info::prelude::{ vec::Vec, fmt::Debug };
use sp_runtime::{ FixedPointOperand, traits::Convert };
use frame_support::traits::{
	Currency,
	LockableCurrency,
	RewardAvailable,
	ValidatorSet,
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
			Copy +
			MaybeSerializeDeserialize +
			Debug +
			MaxEncodedLen +
			TypeInfo +
			FixedPointOperand;

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
		u128,
		ValueQuery
	>;
	/// The aggregate count of rewards presently available for allocation to validators and nominators
	#[pallet::storage]
	#[pallet::getter(fn total_reward)]
	pub type TotalReward<T: Config> = StorageValue<_, u128>;

	/// Specifics regarding the rewards distributed within the designated era
	#[pallet::storage]
	#[pallet::getter(fn era_reward_accounts)]
	pub type EraRewardAccounts<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		u128,
		OptionQuery
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

impl<T: Config> RewardAvailable<T::Balance> for Pallet<T> {
	fn reward_available() -> T::Balance {
		let era_points = ErasRewardPoints::<T>::get(Self::current_era());
		let total_points = era_points.total as u128;
		let total = 000_021_000_000_000_000 * total_points;
		total.into()
	}
}

impl<T: Config> Rewards<T::AccountId> for Pallet<T> {
	fn reward_account() -> Vec<T::AccountId> {
		let account = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		account
	}
	fn claim_rewards(account: T::AccountId) -> DispatchResult {
		let reward = EraRewardAccounts::<T>::get(account.clone()).unwrap_or(0);
		log::info!("here is the reward {}",reward);
		let nominators = Self::check_nominators(account.clone());
		if nominators.is_empty() {
			Self::distribute_reward(account.clone())?;
			Self::recalculate_reward(reward)?;
			Self::recalculate_rewarded_accounts(account.clone())?;
			return Ok(());
		}
		nominators.iter().for_each(|nominator| {
			let _ = Self::distribute_reward(account.clone());
			let _ = Self::distribute_reward(nominator.who.clone());
		});

		Self::recalculate_reward(reward)?;
		Self::recalculate_rewarded_accounts(account.clone())?;
		return Ok(());
	}

	fn calculate_reward() -> DispatchResult {
		let all_validators = T::Validators::validators();

		all_validators.iter().for_each(|validator_id| {
			let validator = T::ValidatorId::convert(validator_id.clone()).unwrap();
			let active_era = pallet_staking::Pallet::<T>::active_era().unwrap();
			let era = active_era.index;
			let era_reward_points = <ErasRewardPoints<T>>::get(era);
			let validator_points = era_reward_points.individual.get(&validator).unwrap_or(&0);
			let exposure = ErasStakers::<T>::get(Self::current_era(), validator.clone());
			let nominators = exposure.others;
			let reward :f64 = 0.021;
			let total_reward = reward as f64 * (*validator_points as f64);
			if nominators.is_empty() {
				let converted_reward = Self::convert_f64_to_u128(total_reward);
				log::info!("here is the converted_reward{}",converted_reward);
				let _ = Self::add_reward(validator.clone(), converted_reward);
			}
			let validator_prefs = Validators::<T>::get(validator.clone());
			let validator_commission = validator_prefs.commission.deconstruct() as f64;
			let nominator_share = (total_reward as f64 * validator_commission as f64) / 100.0;
			let validator_share = total_reward - nominator_share;
			let converted_validator_reward = Self::convert_f64_to_u128(validator_share);
			let _ = Self::add_reward(validator.clone(), converted_validator_reward);
			nominators.iter().for_each(|nominator| {
				let nominator_stake = nominator.value;
				let total_stake = exposure.total;
				let nominator_reward = Self::calculate_nominator_reward(
					nominator_stake.into(),
					total_stake.into(),
					nominator_share
				);
				let converted_reward = Self::convert_f64_to_u128(nominator_reward);
				let _ = Self::add_reward(nominator.who.clone(), converted_reward);
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
		let precision = 18;
		let scaled_share = amount / (10u128).pow(precision).into();
		T::RewardCurrency::transfer(&who, &dest, scaled_share, existence_requirement)?;
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

	fn recalculate_reward(block_reward: u128) -> DispatchResult {
		TotalReward::<T>::mutate(|total_transaction| {
			*total_transaction = Some(total_transaction.unwrap_or(0) - block_reward);
		});
		Ok(())
	}

	fn recalculate_rewarded_accounts(account: T::AccountId) -> DispatchResult {
		let mut era_reward_accounts = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		if let Some(index) = era_reward_accounts.iter().position(|a| a == &account.clone()) {
			era_reward_accounts.remove(index);
		}
		EraRewardsVault::<T>::put(era_reward_accounts);
		Ok(())
	}

	pub fn calculate_nominator_reward(share: u128, total_stake: u128, reward: f64) -> f64 {
		let precision = 18;
		let scaled_share = share / (10u128).pow(precision);
		let scaled_total_stake: u64 = (total_stake / (10u128).pow(precision)) as u64;
		let division: f64 = ((scaled_share as f64) / (scaled_total_stake as f64)) as f64;
		let scaled_reward: f64 = ((reward as f64) / ((10u128).pow(precision) as f64)) as f64;
		let total_reward = division * scaled_reward;
		total_reward
	}

	pub fn convert_f64_to_u128(value: f64) -> u128 {
		let precision = 18;
		let multiplier = (10u128).pow(precision);
		let number = (value * (multiplier as f64)) as u128;
		number
	}

	fn add_reward(account: T::AccountId, reward: u128) -> DispatchResult {
		let earlier_reward = EraRewardAccounts::<T>::get(account.clone()).unwrap_or(0);
		let new_individual_reward = reward + earlier_reward;
		EraRewardAccounts::<T>::insert(account.clone(), new_individual_reward);
		Ok(())
	}

	fn distribute_reward(account: T::AccountId) -> DispatchResult {
		let reward = EraRewardAccounts::<T>::get(account.clone()).unwrap_or(0);
		log::info!("here is the reward {}",reward);
		Self::transfer(
			Self::treasury_account(),
			account.clone(),
			reward.into(),
			KeepAlive
		)?;
		log::info!("here is the transferred");
		EraRewardAccounts::<T>::remove(account.clone());
		BeneficialRewardRecord::<T>::insert(account.clone(), reward);
		Ok(())
	}

	fn current_era() -> u32 {
		CurrentEra::<T>::get().unwrap_or(0)
	}

	fn check_nominators(
		who: T::AccountId
	) -> Vec<IndividualExposure<T::AccountId, <T as pallet_staking::Config>::CurrencyBalance>> {
		let current_era = Self::current_era();
		let exposure = ErasStakers::<T>::get(current_era, who.clone());
		let nominators = exposure.others;
		nominators
	}
}
