// Reward Pallet
// The Reward Pallet is a module in the Substrate blockchain framework designed to manage and distribute rewards to participants based on their contributions within the network.
// This pallet facilitates the allocation of rewards to validators and nominators for their involvement in staking activities.

#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::pallet_prelude::DispatchResult;
use frame_support::ensure;
pub use pallet::*;
use pallet_staking::{ CurrentEra, IndividualExposure, ErasStakers };
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
	reward::RewardAccount,
	ExistenceRequirement,
	ExistenceRequirement::KeepAlive,
};
use frame_election_provider_support::{ ElectionDataProvider, DataProviderBounds };
use sp_staking::EraIndex;

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
			BlockNumber = BlockNumberFor<Self>
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
	/// The count of transaction conducted by Author
	#[pallet::storage]
	#[pallet::getter(fn author_list)]
	pub type AuthorBlockList<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u128>;

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

	/// The number of the transactions executed in one era
	/// Specifics regarding the rewards distributed within the designated era
	#[pallet::storage]
	#[pallet::getter(fn transaction_per_era)]
	pub type TransactionPerEra<T: Config> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		EraIndex,
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
		/// The available rewards for the validator's rewards.
		ValidatorRewardAvailable {
			who: T::AccountId,
			balance:u128,
		},
		/// The available reward for the nominator of the specific validator.
		NominatorRewardAvailable {
			who: T::AccountId,
			balance:u128,
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
			Self::deposit_event(Event::<T>::Rewarded{who:validator});
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(Weight::zero())]
		pub fn check_validator_reward(
			origin: OriginFor<T>,
			validator: T::AccountId
		) -> DispatchResult {
			ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let reward = EraRewardAccounts::<T>::get(validator.clone()).ok_or(Error::<T>::NoRewardAvailable)?;
			Self::deposit_event(Event::ValidatorRewardAvailable {
				who: validator.clone(),
				balance: reward,
			});
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::zero())]
		pub fn check_nominator_reward(
			origin: OriginFor<T>,
			validator: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::verify_validator(validator.clone())?;
			let reward = EraRewardAccounts::<T>::get(who).ok_or(Error::<T>::NoRewardAvailable)?;
			Self::deposit_event(Event::NominatorRewardAvailable {
				who: validator.clone(),
				balance: reward,
			});
			Ok(())
		}
	}
}

impl<T: Config> RewardAvailable<T::Balance> for Pallet<T> {
	fn reward_available() -> T::Balance {
		let reward = TotalReward::<T>::get().unwrap_or(0);
		let total = 009_000_000_000_000_000 * reward;
		total.into()
	}
}

impl<T: Config> RewardAccount<T::AccountId> for Pallet<T> {
	fn reward_account() -> Vec<T::AccountId> {
		let account = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
		account
	}
	fn claim_rewards(who: T::AccountId) -> DispatchResult {
		let block_reward = AuthorBlockList::<T>
			::get(who.clone())
			.ok_or(Error::<T>::NoRewardAvailable)?;
			let nominators = Self::check_nominators(who.clone());

			if nominators.is_empty() {
			Self::distribute_validator_reward(who.clone())?;
			Self::recalculate_reward(block_reward)?;
			Self::recalculate_rewarded_accounts(who.clone())?;
			AuthorBlockList::<T>::remove(who.clone());
			return Ok(());
			}

			nominators.iter().for_each(|nominator| {
			let _ = Self::distribute_validator_reward(who.clone());
			let nominator_reward = EraRewardAccounts::<T>::get(nominator.who.clone()).unwrap_or(0);
			let _ = Self::transfer(
				Self::treasury_account(),
				nominator.who.clone(),
				nominator_reward.into(),
				KeepAlive
			);
			BeneficialRewardRecord::<T>::insert(nominator.who.clone(), nominator_reward);
		});

		Self::recalculate_reward(block_reward)?;
		AuthorBlockList::<T>::remove(who.clone());
		Self::recalculate_rewarded_accounts(who.clone())?;
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
			ensure!(AuthorBlockList::<T>::contains_key(&who), Error::<T>::NoRewardAvailable);
		} else {
			ensure!(AuthorBlockList::<T>::contains_key(&who), Error::<T>::NoSuchValidator);
		}
		Ok(())
	}

	fn recalculate_reward(block_reward: u128) -> DispatchResult    {
		TotalReward::<T>::mutate(|total_transaction| {
			*total_transaction = Some(total_transaction.unwrap_or(0) - block_reward);
		});
		Ok(())
	}
	
	fn recalculate_rewarded_accounts(account: T::AccountId) -> DispatchResult{
		let mut era_reward_accounts = EraRewardsVault::<T>::get().unwrap_or_else(Vec::new);
			if let Some(index) = era_reward_accounts.iter().position(|a| a == &account.clone()) {
				era_reward_accounts.remove(index);
			}
			EraRewardsVault::<T>::put(era_reward_accounts);
			Ok(())
	}

	fn distribute_validator_reward(account:T::AccountId) -> DispatchResult{
		let validator_reward = EraRewardAccounts::<T>::get(account.clone()).unwrap_or(0);
		Self::transfer(Self::treasury_account(), account.clone(), validator_reward.into(), KeepAlive)?;
		EraRewardAccounts::<T>::remove(account.clone());
		BeneficialRewardRecord::<T>::insert(account.clone(), validator_reward);
		Ok(())
	}

	fn check_nominators(
		who: T::AccountId
	) -> Vec<IndividualExposure<T::AccountId, <T as pallet_staking::Config>::CurrencyBalance>> {
		let current_era = CurrentEra::<T>::get().unwrap_or(0);
		let exposure = ErasStakers::<T>::get(current_era, who.clone());
		let nominators = exposure.others;
		nominators
	}
}
