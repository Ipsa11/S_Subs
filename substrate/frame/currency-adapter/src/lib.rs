//! # Currency adapter pallet
//!
//! ## Overview
//!
//! This pallet works like a bridge between pallet-balances & pallet-assets

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
use frame_system::pallet_prelude::BlockNumberFor;
use frame_support::{
    defensive,
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{
        tokens::{
            fungible::{Inspect, Mutate},
            fungibles::{Dust, Inspect as Inspects, Mutate as Mutates, Unbalanced as Unbalanceds},
            DepositConsequence, Fortitude, Precision, Preservation, Provenance,
            WithdrawConsequence,
        },
        Get, LockIdentifier, WithdrawReasons,
    },
};
use liquid_staking_primitives::{Balance, CurrencyId};
use sp_runtime::DispatchError;

type AssetIdOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::AssetId;
type BalanceOf<T> =
    <<T as Config>::Assets as Inspects<<T as frame_system::Config>::AccountId>>::Balance;

const CURRENCY_ADAPTER_ID: LockIdentifier = *b"cadapter";

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::traits::LockableCurrency;
    use frame_system::pallet_prelude::OriginFor;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Assets: Inspects<Self::AccountId, AssetId = CurrencyId, Balance = Balance>
            + Mutates<Self::AccountId, AssetId = CurrencyId, Balance = Balance>;

        type Balances: Inspect<Self::AccountId, Balance = Balance>
            + Mutate<Self::AccountId, Balance = Balance>
            + LockableCurrency<Self::AccountId, Balance = Balance, Moment = BlockNumberFor<Self>>;

        #[pallet::constant]
        type GetNativeCurrencyId: Get<AssetIdOf<Self>>;

        // Origin which can lock asset balance
        type LockOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::error]
    pub enum Error<T> {
        /// Not a native token
        NotANativeToken,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::zero())]
        pub fn force_set_lock(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            who: T::AccountId,
            #[pallet::compact] amount: BalanceOf<T>,
        ) -> DispatchResult {
            T::LockOrigin::ensure_origin(origin)?;
            ensure!(
                asset == T::GetNativeCurrencyId::get(),
                Error::<T>::NotANativeToken
            );
            T::Balances::set_lock(CURRENCY_ADAPTER_ID, &who, amount, WithdrawReasons::all());
            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(Weight::zero())]
        pub fn force_remove_lock(
            origin: OriginFor<T>,
            asset: AssetIdOf<T>,
            who: T::AccountId,
        ) -> DispatchResult {
            T::LockOrigin::ensure_origin(origin)?;
            ensure!(
                asset == T::GetNativeCurrencyId::get(),
                Error::<T>::NotANativeToken
            );
            T::Balances::remove_lock(CURRENCY_ADAPTER_ID, &who);
            Ok(())
        }
    }
}

impl<T: Config> Inspects<T::AccountId> for Pallet<T> {
    type AssetId = AssetIdOf<T>;
    type Balance = BalanceOf<T>;

    fn total_issuance(asset: Self::AssetId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::total_issuance()
        } else {
            T::Assets::total_issuance(asset)
        }
    }

    fn minimum_balance(asset: Self::AssetId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::minimum_balance()
        } else {
            T::Assets::minimum_balance(asset)
        }
    }

    fn total_balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::total_balance(who)
        } else {
            T::Assets::total_balance(asset, who)
        }
    }

    fn balance(asset: Self::AssetId, who: &T::AccountId) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::balance(who)
        } else {
            T::Assets::balance(asset, who)
        }
    }

    fn reducible_balance(
        asset: Self::AssetId,
        who: &T::AccountId,
        preservation: Preservation,
        force: Fortitude,
    ) -> Self::Balance {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::reducible_balance(who, preservation, force)
        } else {
            T::Assets::reducible_balance(asset, who, preservation, force)
        }
    }

    fn can_deposit(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        mint: Provenance,
    ) -> DepositConsequence {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::can_deposit(who, amount, mint)
        } else {
            T::Assets::can_deposit(asset, who, amount, mint)
        }
    }

    fn can_withdraw(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> WithdrawConsequence<Self::Balance> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::can_withdraw(who, amount)
        } else {
            T::Assets::can_withdraw(asset, who, amount)
        }
    }
    fn asset_exists(asset: Self::AssetId) -> bool {
        if asset == T::GetNativeCurrencyId::get() {
            true
        } else {
            T::Assets::asset_exists(asset)
        }
    }
}

impl<T: Config> Mutates<T::AccountId> for Pallet<T> {
    fn mint_into(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
    ) -> Result<Self::Balance, DispatchError> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::mint_into(who, amount)
        } else {
            T::Assets::mint_into(asset, who, amount)
        }
    }

    fn burn_from(
        asset: Self::AssetId,
        who: &T::AccountId,
        amount: Self::Balance,
        precision: Precision,
        force: Fortitude,
    ) -> Result<Self::Balance, DispatchError> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::burn_from(who, amount, precision, force)
        } else {
            T::Assets::burn_from(asset, who, amount, precision, force)
        }
    }

    fn transfer(
        asset: Self::AssetId,
        source: &T::AccountId,
        dest: &T::AccountId,
        amount: Self::Balance,
        preservation: Preservation,
    ) -> Result<Self::Balance, DispatchError> {
        if asset == T::GetNativeCurrencyId::get() {
            T::Balances::transfer(source, dest, amount, preservation)
        } else {
            T::Assets::transfer(asset, source, dest, amount, preservation)
        }
    }
}

impl<T: Config> Unbalanceds<T::AccountId> for Pallet<T> {
    fn handle_dust(_: Dust<T::AccountId, Self>) {
        defensive!("`decrease_balance` and `increase_balance` have non-default impls; nothing else calls this; qed");
    }
    fn write_balance(
        _: Self::AssetId,
        _: &T::AccountId,
        _: Self::Balance,
    ) -> Result<Option<Self::Balance>, DispatchError> {
        defensive!("write_balance is not used");
        Err(DispatchError::Unavailable)
    }
    fn set_total_issuance(_: AssetIdOf<T>, _: Self::Balance) {
        defensive!("set_total_issuance is not used");
    }
}
