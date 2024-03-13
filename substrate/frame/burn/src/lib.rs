// Burn Pallet 
// The Burn Pallet in the Substrate blockchain framework provides functionality for burning or destroying a specified amount of native tokens within the system.
// This pallet allows for the removal of tokens from circulation, typically as part of a deflationary economic model

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


use frame_support::traits::{ReservableCurrency,Currency};
use frame_support::PalletId;
use sp_runtime::traits::AccountIdConversion;
pub use pallet::*;
type BurnBalance<T, I> = <<T as Config<I>>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type RuntimeEvent: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type PalletId: Get<PalletId>;
		type BurnOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}
	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The Balance has been burned
		  Burned {amount:BurnBalance<T, I> },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Balance too low to send value.
		InsufficientBalance
	}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		#[serde(skip)]
		_config: sp_std::marker::PhantomData<(T, I)>,
	}
	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {
			  <Pallet<T, I>>::account_id();
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn burn_account() -> Option<T::AccountId>{
			let account = Self::account_id();
			Some(account)
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::zero())]
		pub fn burn_amount(origin: OriginFor<T>, amount: BurnBalance<T, I>) -> DispatchResult {
		 	 T::BurnOrigin::ensure_origin(origin)?;
			let balance = T::Currency::total_balance(&Self::account_id());
			 ensure!(amount < balance,Error::<T,I>::InsufficientBalance );
			 T::Currency::slash(&Self::account_id(), amount);
			  Self::deposit_event(Event::Burned{amount :amount});
			Ok(())
		}
	}
}
