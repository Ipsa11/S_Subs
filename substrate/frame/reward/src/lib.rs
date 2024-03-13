// Reward Pallet
// The Reward Pallet is a module in the Substrate blockchain framework designed to manage and distribute rewards to participants based on their contributions within the network.
// This pallet facilitates the allocation of rewards to validators and nominators for their involvement in staking activities.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::DispatchResult;
pub use pallet::*;
use pallet_staking::{ CurrentEra, ErasStakers };
use pallet_treasury::TreasuryAccountId;
use sp_runtime::traits::AtLeast32BitUnsigned;
use parity_scale_codec::Codec;
use scale_info::prelude::fmt::Debug;
use sp_runtime::FixedPointOperand;
use frame_support::traits::reward::RewardAccount;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::{
	Currency,
	LockableCurrency,
	RewardAvailable,
	ValidatorSet,
	ExistenceRequirement,
	ExistenceRequirement::KeepAlive,
};
use frame_election_provider_support::{ ElectionDataProvider, DataProviderBounds };
use sp_runtime::traits::Convert;
use scale_info::prelude::vec::Vec;

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
		type DataProvider: ElectionDataProvider<
			AccountId = <Self::ValidatorSet as ValidatorSet<Self::AccountId>>::ValidatorId,
			BlockNumber = BlockNumberFor<Self>>;

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
	/// The count of transaction conducted by Author
	#[pallet::storage]
	#[pallet::getter(fn author_list)]
	pub type AuthorBlockList<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128>;

	/// The era reward given to the validator
	#[pallet::storage]
	#[pallet::getter(fn total_rewards)]
	pub type TotalRewards<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, T::Balance,ValueQuery>;

	/// Total numbers of reward
	#[pallet::storage]
	#[pallet::getter(fn total_reward)]
	pub type TotalReward<T: Config> = StorageValue<_, u128>;

	/// Era reward
	#[pallet::storage]
	#[pallet::getter(fn era_reward)]
	pub type EraReward<T: Config> = StorageValue<_, Vec<T::AccountId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The rewards has been succesfully distributed
		Rewarded {
			who: T::AccountId,
			balance: T::Balance,
		},
		/// The available rewards for the validator's rewards.
		ValidatorRewardAvailable{
			who:T::AccountId,
			balance:T::Balance,
		},
		/// The available reward for the nominator of the specific validator.
		NominatorRewardAvailable{
			who:T::AccountId,
			balance:T::Balance,
		}
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
		NoSuchNominator
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn get_rewards(origin: OriginFor<T>,validator:T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let mut era_reward_accounts = EraReward::<T>::get().unwrap_or_else(Vec::new);
			ensure!(!era_reward_accounts.contains(&validator), Error::<T>::AlreadyRewarded);
			era_reward_accounts.push(validator.clone());
			EraReward::<T>::put(era_reward_accounts);
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn check_validator_reward(origin: OriginFor<T>,validator:T::AccountId) -> DispatchResult {
			ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let reward = AuthorBlockList::<T>::get(validator.clone()).unwrap();
			let total_reward = reward * 25_000_000_000_000_000_000;
			let nominators = Self::check_nominators(validator.clone());
			if nominators.is_empty() {
				Self::deposit_event(Event::ValidatorRewardAvailable { who: validator.clone(), balance: total_reward.into() });
			}
			let total_recipients = 1 + (nominators.len() as u128);
			let individual_share: u128 = total_reward / total_recipients;
			Self::deposit_event(Event::ValidatorRewardAvailable { who: validator.clone(), balance: individual_share.into() });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())]
		pub fn check_nominator_reward(origin: OriginFor<T>,validator:T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let nominators = Self::check_nominators(validator.clone());
			if nominators.is_empty() {
				return Err(Error::<T>::NoSuchNominator.into());
			}
			for i in &nominators{
				ensure!(who == i.who, Error::<T>::NoSuchNominator);
			}
			let reward = AuthorBlockList::<T>::get(validator.clone()).unwrap();
			let total_reward = reward * 25_000_000_000_000_000_000;
			let total_recipients = 1 + (nominators.len() as u128);
			let individual_share: u128 = total_reward / total_recipients;
			Self::deposit_event(Event::NominatorRewardAvailable { who: validator.clone(), balance: individual_share.into() });
			Ok(())
		}
	}
}

impl<T: Config> RewardAvailable<T::Balance> for Pallet<T> {
	fn reward_available() -> T::Balance {
		let reward = TotalReward::<T>::get().unwrap_or(0);
		let total = 25_000_000_000_000_000_000 * reward;
		total.into()
	}
}

impl<T: Config> RewardAccount<T::AccountId> for Pallet<T> {
	fn reward_account() -> Vec<T::AccountId> {
		let account = EraReward::<T>::get().unwrap_or_else(Vec::new);
		account
	}
	fn claim_rewards(who: T::AccountId) -> Result<(), DispatchError> {
		let block_reward = AuthorBlockList::<T>::get(who.clone()).unwrap();
		let total:T::Balance = (25_000_000_000_000_000_000 * block_reward).into();
		let nominators = Self::check_nominators(who.clone());

		if nominators.is_empty() {
		 	Self::transfer(Self::treasury_account(), who.clone(), total.into(), KeepAlive)?;
			TotalRewards::<T>::insert(who.clone(),total);
			Self::recalculate_reward(block_reward);
			AuthorBlockList::<T>::remove(who.clone());
			let mut era_reward_accounts = EraReward::<T>::get().unwrap_or_else(Vec::new);
			if let Some(index) = era_reward_accounts.iter().position(|a| a == &who.clone()) {
				era_reward_accounts.remove(index);
			}
			EraReward::<T>::put(era_reward_accounts);
			return Ok(());
		}

		let total_recipients = 1 + (nominators.len() as u128);
		let individual_share: T::Balance = total / total_recipients.into();
		Self::transfer(Self::treasury_account(), who.clone(), individual_share.into(), KeepAlive)?;
		TotalRewards::<T>::insert(who.clone(),individual_share);

		for i in nominators {
			Self::transfer(Self::treasury_account(), i.who.clone(), individual_share.into(), KeepAlive)?;
			TotalRewards::<T>::insert(i.who.clone(),individual_share);
		}

		Self::recalculate_reward(block_reward);
		AuthorBlockList::<T>::remove(who.clone());
		let mut era_reward_accounts = EraReward::<T>::get().unwrap_or_else(Vec::new);
		if let Some(index) = era_reward_accounts.iter().position(|a| a == &who.clone()) {
			era_reward_accounts.remove(index);
		}
		EraReward::<T>::put(era_reward_accounts);
		return Ok(());
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
		Self::deposit_event(Event::Rewarded { who: dest, balance: amount });
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
			frame_support::ensure!(
				AuthorBlockList::<T>::contains_key(&who),
				Error::<T>::NoRewardAvailable
			);
		} else {
			frame_support::ensure!(
				AuthorBlockList::<T>::contains_key(&who),
				Error::<T>::NoSuchValidator
			);
		}
		Ok(())
	}

	fn recalculate_reward(block_reward: u128) {
		TotalReward::<T>::mutate(|total_transaction| {
			*total_transaction = Some(total_transaction.unwrap_or(0) - block_reward);
		});
	}

	fn check_nominators(
		who: T::AccountId
	) -> Vec<
		pallet_staking::IndividualExposure<
			T::AccountId,
			<T as pallet_staking::Config>::CurrencyBalance
		>
	> {
		let current_era = CurrentEra::<T>::get().unwrap();
		let exposure = ErasStakers::<T>::get(current_era, who.clone());
		let nominators = exposure.others;
		nominators
	}
}
