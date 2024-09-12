// Reward Pallet
// The Reward Pallet is a module in the Substrate blockchain framework designed to manage and
// distribute rewards to participants based on their contributions within the network. This pallet
// facilitates the allocation of rewards to validators and nominators for their involvement in
// staking activities.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_election_provider_support::{DataProviderBounds, ElectionDataProvider};
use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{
		liquid_staking::StakingAccount, reward::Rewards, Currency, ExistenceRequirement,
		ExistenceRequirement::KeepAlive, Get, LockableCurrency, ValidatorSet,
	},
};
use pallet_staking::BalanceOf;
pub use pallet::*;
use pallet_staking::{CurrentEra, ErasRewardPoints, ErasStakers, Exposure, IndividualExposure, Validators};
use pallet_treasury::TreasuryAccountId;
use parity_scale_codec::Codec;
use scale_info::prelude::{fmt::Debug, vec::Vec};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Convert, Zero},
	FixedPointOperand,
};
use frame_support::ensure;

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
		type LiquidStakeVault: StakingAccount<Self::AccountId>;
		type ValidatorId: Convert<
			<<Self as Config>::Validators as ValidatorSet<
				<Self as frame_system::Config>::AccountId,
			>>::ValidatorId,
			Option<Self::AccountId>,
		>;
		type Balance: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Codec
			+ Default
			+ From<u128>
			+ Into<u128>
			+ Copy
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaxEncodedLen
			+ TypeInfo
			+ FixedPointOperand;

		type Precision: Get<u32>;

		#[pallet::constant]
		type TotalMinutesPerYear: Get<u128>;
		type EraMinutes: Get<u128>;
		type TreasuryAccount: TreasuryAccountId<Self::AccountId>;
		type RewardCurrency: LockableCurrency<
			Self::AccountId,
			Moment = BlockNumberFor<Self>,
			Balance = Self::Balance,
		>;
	}

	/// The era reward which are distributed among the validator and nominator
	#[pallet::storage]
	#[pallet::getter(fn total_rewards)]
	pub type BeneficialRewardRecord<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// Specifies the total nominators of the validator
	#[pallet::storage]
	#[pallet::getter(fn era_reward)]
	pub type EraReward<T: Config> = 
	StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>,ValueQuery>;	

	/// Specifics regarding the rewards distributed within the designated era of the validator
	#[pallet::storage]
	#[pallet::getter(fn validator_reward_accounts)]
	pub type ValidatorRewardAccounts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance, ValueQuery>;

	/// Specifics regarding the rewards distributed within the designated era of the nominator
	#[pallet::storage]
	#[pallet::getter(fn nominator_reward_accounts)]
	pub type NominatorRewardAccounts<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, T::AccountId, Blake2_128Concat,T::AccountId, T::Balance, ValueQuery>;

	/// Era reward accounts
	#[pallet::storage]
	#[pallet::getter(fn era_reward_vault)]
	pub type EraRewardsVault<T: Config> = StorageValue<_, Vec<T::AccountId>>;

	// Storage for the default value
	#[pallet::storage]
	pub type BaseRewardPercent<T> = StorageValue<_, u32, ValueQuery, DefaultVal>;

	// Storage for the mutable value
	#[pallet::storage]
	pub type RewardPercent<T> = StorageValue<_, u32>;

	// Define the default value
	pub struct DefaultVal;
	impl Get<u32> for DefaultVal {
		fn get() -> u32 {
			8
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The rewards has been succesfully distributed
		Distributed { who: T::AccountId, balance: T::Balance },
		/// The reward will be distributed after completely the era
		Rewarded { who: T::AccountId },
		/// The storage value has been set or updated.
		ValueSet { value: u32 },
		/// Insufficient Reward Balance
		InsufficientRewardBalance
	}

	#[pallet::error]
	pub enum Error<T> {
		/// No reward is present
		NoReward,
		/// Wait for the era to complete
		WaitTheEraToComplete,
		/// Insufficient Reward Balance
		InsufficientRewardBalance,
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

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn set_reward_percent_value(origin: OriginFor<T>, value: u32) -> DispatchResult {
			ensure_root(origin)?;
			RewardPercent::<T>::put(value);
			Self::deposit_event(Event::ValueSet { value });
			Ok(())
		}
	}
}

impl<T: Config> Rewards<T::AccountId> for Pallet<T> {
	/// List of the validators who will recieve reward after the era
	fn payout_validators() -> Vec<T::AccountId> {
		let validators = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		validators
	}

	/// Distributing rewards to validators and nominators.
	fn claim_rewards(validator: T::AccountId) -> DispatchResult {
		let nominators = EraReward::<T>::get(validator.clone());
		let reward_balance = Self::verify_balance(validator.clone());
		if let Err(e) = reward_balance {
			Self::deposit_event(Event::InsufficientRewardBalance);
			return Err(e);
		}
		Self::distribute_reward(validator.clone(), None)?;
		Self::update_rewarded_accounts(validator.clone())?;
		if nominators.is_empty() {
			return Ok(());
		}
		nominators.iter().for_each(|nominator| {
			let _ = Self::distribute_reward(validator.clone(), Some(nominator.clone()));
		});
		EraReward::<T>::remove(validator.clone());
		return Ok(());
	}


	/// Function for computing the rewards of validators and nominators at the end of each era
	fn calculate_reward() -> DispatchResult {
		let validators = T::Validators::validators();

		validators.iter().for_each(|validator_id| {
			let validator = T::ValidatorId::convert(validator_id.clone()).unwrap();
			let era_validator_points = Self::retrieve_validator_point(validator.clone());
			let validator_exposure= ErasStakers::<T>::get(Self::current_era(), validator.clone());
			let annual_validator_stake_reward = Self::calculate_annual_validator_reward(validator_exposure.clone());
			let total_era_reward = Self::compute_era_reward_from_annual(annual_validator_stake_reward.into());
			let validator_era_reward = Self::calculate_validator_era_reward(era_validator_points.into(), total_era_reward as f64);
			let nominators = Self::check_nominators(validator.clone());
			if nominators.is_empty() {
				Self::allocate_rewards(
					validator.clone(),
					None,
					(validator_era_reward as u128).into(),
				);
				return;
			}
			let (total_validator_reward,remaining_reward_for_nominators) = Self::calculate_validator_commission_reward(validator.clone(),validator_era_reward,validator_exposure.clone());
			Self::allocate_rewards(
				validator.clone(),
				None,
				(total_validator_reward as u128).into(),
			);
			if remaining_reward_for_nominators.is_zero(){
				return;
			}
			nominators.iter().for_each(|nominator| {
				let mut current_nominators = EraReward::<T>::get(validator.clone());
				if !current_nominators.contains(&nominator.who.clone()) {
					current_nominators.push(nominator.who.clone());
					EraReward::<T>::insert(validator.clone(), current_nominators);
				}
				let nominator_stake = nominator.value;
				let nominator_reward = Self::calculate_reward_share(
					nominator_stake.into(),
					validator_exposure.total.into(),
					remaining_reward_for_nominators.into(),
				);
				Self::allocate_rewards(
					validator.clone(),
					Some(nominator.who.clone()),
					(nominator_reward as u128).into(),
				);
			});
		});
		Ok(())
	}

	fn reward_percent() -> DispatchResult {
		let new_reward_percent = RewardPercent::<T>::get().unwrap_or(8);
		BaseRewardPercent::<T>::put(new_reward_percent);
		Ok(())
	}
}

impl<T: Config> Pallet<T> {

	/// Transfer an amount to the accounts with respecting the `keep_alive` requirements.
	fn transfer(
		who: T::AccountId,
		dest: T::AccountId,
		amount: T::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		T::RewardCurrency::transfer(&who, &dest, amount, existence_requirement)?;
		Self::deposit_event(Event::Distributed { who: dest, balance: amount });
		Ok(())
	}

	/// Rewards will be disbursed from the treasury account
	fn treasury_account() -> T::AccountId {
		T::TreasuryAccount::accountid()
	}

	/// Validation of an account to determine its validator status.
	fn verify_validator(validator: T::AccountId) -> DispatchResult {
		let validator = ValidatorRewardAccounts::<T>::get(validator);
		Self::check_reward(validator)
	}

	/// Verify the balance of reward 
	fn verify_balance(validator: T::AccountId) -> DispatchResult {
		let validator_reward = ValidatorRewardAccounts::<T>::get(validator.clone());
		let free_balance =T::RewardCurrency::free_balance(&Self::treasury_account());
		let nominators = EraReward::<T>::get(validator.clone());
		if nominators.is_empty(){
			ensure!(free_balance >= validator_reward,Error::<T>::InsufficientRewardBalance);
			return Ok(());
		}
		let mut total_nominator_reward:T::Balance = 0u128.into();
		nominators.iter().for_each(|nominator| {
			let nominator_reward = NominatorRewardAccounts::<T>::get(validator.clone(),nominator);
			total_nominator_reward += nominator_reward;
		});
		ensure!(free_balance >= total_nominator_reward + validator_reward,Error::<T>::InsufficientRewardBalance);
		return Ok(());
	}
	

	/// Check if the reward exist
	fn check_reward(reward: T::Balance) -> DispatchResult {
		if reward.is_zero() {
			return Err(Error::<T>::NoReward.into());
		} else {
			return Ok(());
		}
	}

	/// Calculates the total reward for a validator based on their commission and stake, as well as the remaining reward for nominators.
	fn calculate_validator_commission_reward(validator: T::AccountId, validator_era_reward: f64, exposure : Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>,) -> (f64,f64) {
		let validator_commission = Self::validator_commission(validator.clone());
		let validator_share = ((validator_era_reward as f64) * (validator_commission as f64)) / 100.0;
		let validator_stake = exposure.own;
		let remaining_reward = validator_era_reward - validator_share;
		let validator_own_share_reward = Self::calculate_reward_share(
			validator_stake.into(),
			exposure.total.into(),
			remaining_reward,
		);
		let total_validator_reward = validator_share + validator_own_share_reward;
		(total_validator_reward,remaining_reward)
	}

	/// Get the commission for the validator.
	fn validator_commission(validator: T::AccountId) -> u32 {
		let validator_prefs = Validators::<T>::get(validator.clone());
		let validator_commission = validator_prefs.commission.deconstruct();
		let precision: u32 = 7;
		let scaled_commission: u32 = validator_commission / (10u32).pow(precision);
		scaled_commission
	}

	/// Update the list of validators who have already been rewarded.
	fn update_rewarded_accounts(account: T::AccountId) -> DispatchResult {
		let mut era_reward_accounts = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		if let Some(index) = era_reward_accounts.iter().position(|a| a == &account.clone()) {
			era_reward_accounts.remove(index);
		}
		EraRewardsVault::<T>::put(era_reward_accounts);
		Ok(())
	}

	/// Compute the annual reward of the validator's total stake for distribution
	pub fn calculate_annual_validator_reward(exposure : Exposure<<T as frame_system::Config>::AccountId, BalanceOf<T>>) -> u128 {
		let validator_stake = exposure.total;
		let divisor: T::CurrencyBalance = 100u128.into();
		let reward_percent = (validator_stake * BaseRewardPercent::<T>::get().into()) / divisor;
		reward_percent.into()
	}

	/// Compute the total era reward allotted to the validator
	pub fn compute_era_reward_from_annual(reward_percent: u128) -> f64 {
		let total_minutes_per_year = T::TotalMinutesPerYear::get();
		let era_minutes = T::EraMinutes::get();
		let era = total_minutes_per_year / era_minutes;
		let total_reward = reward_percent;
		let era_reward = total_reward as f64 / era as f64;
		era_reward 
	}

	/// Compute the reward share for a validator and nominator based on their stake.
	pub fn calculate_reward_share(share: u128, total_stake: u128, reward: f64) -> f64 {
		let precision = T::Precision::get();
		let scaled_share = share / (10u128).pow(precision);
		let scaled_total_stake: u64 = (total_stake / (10u128).pow(precision)) as u64;
		let division: f64 = ((scaled_share as f64) / (scaled_total_stake as f64)) as f64;
		let total_reward = division * reward;
		total_reward
	}

	/// Allocates rewards to the specified validator.
	fn allocate_rewards(
		validator: T::AccountId,
		nominator: Option<T::AccountId>,
		reward: T::Balance
	) {
		if let Some(nominator) = nominator {
			NominatorRewardAccounts::<T>::mutate(validator, nominator.clone(), |earlier_reward| {
				*earlier_reward += reward;
			})
		} else {
			ValidatorRewardAccounts::<T>::mutate(validator.clone(), |earlier_reward| {
				*earlier_reward += reward;
			})
		}
	}

	/// Retrieves the points of the validator.
	fn retrieve_validator_point(account: T::AccountId) -> u32 {
		let era_reward_points = <ErasRewardPoints<T>>::get(Self::active_era());
		let validator_points = era_reward_points.individual.get(&account).unwrap_or(&0);
		*validator_points
	}

	/// Distributes rewards to the validator and nominators.
	fn distribute_reward(
		validator: T::AccountId,
		nominator: Option<T::AccountId>
	) -> DispatchResult {
		let (reward, recipient) = if let Some(nominator) = nominator {
			let reward = NominatorRewardAccounts::<T>::get(validator.clone(), nominator.clone());
			Self::check_reward(reward)?;
			Self::transfer(Self::treasury_account(), nominator.clone(), reward, KeepAlive)?;
			let staking_account = T::LiquidStakeVault::staking_account();
			if nominator != staking_account {
				let mut nominators = EraReward::<T>::get(validator.clone());
				if
					let Some(index) = nominators
						.iter()
						.position(|nominator_account| nominator_account == &nominator.clone())
				{
					nominators.remove(index);
				}
				EraReward::<T>::insert(validator.clone(),nominators.clone());
				NominatorRewardAccounts::<T>::remove(validator.clone(), nominator.clone());
			}
			(reward, nominator)
		} else {
			let reward = ValidatorRewardAccounts::<T>::get(validator.clone());
			Self::check_reward(reward)?;
			Self::transfer(Self::treasury_account(), validator.clone(), reward, KeepAlive)?;
			ValidatorRewardAccounts::<T>::remove(validator.clone());
			(reward, validator)
		};
		Self::store_reward_received(recipient, reward);
		Ok(())
	}

	/// Current era index
	fn current_era() -> u32 {
		CurrentEra::<T>::get().unwrap_or(0)
	}

	/// Active era index
	fn active_era() -> u32 {
		let active_era = pallet_staking::Pallet::<T>::active_era().unwrap();
		let era = active_era.index;
		era
	}
	/// Store the received reward for a specific account.
	fn store_reward_received(account: T::AccountId, reward: T::Balance) {
		BeneficialRewardRecord::<T>::mutate(account.clone(), |earlier_reward| {
			*earlier_reward += reward;
		});
	}

	/// Compute the reward of the validator within the era 
	fn calculate_validator_era_reward(validator_points: u32, era_reward: f64) -> f64 {
		let era_reward_points = <ErasRewardPoints<T>>::get(Self::active_era());
		let total_points = era_reward_points.total as u32;
		let reward = ((validator_points as f64) / (total_points as f64)) * (era_reward as f64);
		reward
	}	

	/// Determine whether the validator has nominators in the current era.
	fn check_nominators(
		validator: T::AccountId,
	) -> Vec<IndividualExposure<T::AccountId, <T as pallet_staking::Config>::CurrencyBalance>> {
		let exposure = ErasStakers::<T>::get(Self::current_era(), validator.clone());
		let nominators = exposure.others;
		nominators
	}
}
