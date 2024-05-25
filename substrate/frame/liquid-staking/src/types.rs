use sp_runtime::RuntimeDebug;
use parity_scale_codec::{ Decode, Encode };
use scale_info::TypeInfo;
use crate::Config;
use crate::AssetIdOf;
use sp_runtime::ArithmeticError;
use frame_support::pallet_prelude::DispatchResult;
use crate::Pallet;
use frame_support::pallet_prelude::DispatchError;
use sp_runtime::FixedPointOperand;
use sp_runtime::traits::Get;
use frame_support::traits::tokens::Balance as BalanceT;
#[derive(Copy, Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum Staking {
	Bond,
	NoBond,
}

pub trait DecimalProvider<CurrencyId> {
	fn get_decimal(asset_id: &CurrencyId) -> Option<u8>;
}

pub trait LiquidStakingCurrenciesProvider<CurrencyId> {
	fn get_liquid_currency() -> Option<CurrencyId>;
}

impl<T: Config> LiquidStakingCurrenciesProvider<AssetIdOf<T>> for Pallet<T> {

	fn get_liquid_currency() -> Option<AssetIdOf<T>> {
		let asset_id = T::LiquidCurrency::get();
		if T::Decimal::get_decimal(&asset_id).is_some() {
			Some(asset_id)
		} else {
			None
		}
	}
}

/// The matching pool's total stake & unstake amount in one era
#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MatchingLedger<Balance> {
	/// The total stake amount in one era
	pub total_stake_amount: ReservableAmount<Balance>,
	/// The total unstake amount in one era
	pub total_unstake_amount: ReservableAmount<Balance>,
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ReservableAmount<Balance> {
	pub total: Balance,
	pub reserved: Balance,
}

impl<Balance: BalanceT + FixedPointOperand> ReservableAmount<Balance> {
	pub fn free(&self) -> Result<Balance, DispatchError> {
		Ok(self.total.checked_sub(&self.reserved).ok_or(ArithmeticError::Underflow)?)
	}
}

impl<Balance: BalanceT + FixedPointOperand> MatchingLedger<Balance> {
	pub fn add_stake_amount(&mut self, amount: Balance) -> DispatchResult {
		self.total_stake_amount.total = self.total_stake_amount.total
			.checked_add(&amount)
			.ok_or(ArithmeticError::Overflow)?;
		Ok(())
	}

	pub fn add_unstake_amount(&mut self, amount: Balance) -> DispatchResult {
		self.total_unstake_amount.total = self.total_unstake_amount.total
			.checked_add(&amount)
			.ok_or(ArithmeticError::Overflow)?;
		Ok(())
	}

	pub fn sub_stake_amount(&mut self, amount: Balance) -> DispatchResult {
		let total_free_stake_amount = self.total_stake_amount.free()?;
		if total_free_stake_amount < amount {
			return Err(ArithmeticError::Underflow.into());
		}

		self.total_stake_amount.total = self.total_stake_amount.total
			.checked_sub(&amount)
			.ok_or(ArithmeticError::Underflow)?;
		Ok(())
	}

	pub fn sub_unstake_amount(&mut self, amount: Balance) -> DispatchResult {
		let total_free_unstake_amount = self.total_unstake_amount.free()?;
		if total_free_unstake_amount < amount {
			return Err(ArithmeticError::Underflow.into());
		}

		self.total_unstake_amount.total = self.total_unstake_amount.total
			.checked_sub(&amount)
			.ok_or(ArithmeticError::Underflow)?;
		Ok(())
	}

	pub fn set_stake_amount_lock(&mut self, amount: Balance) -> DispatchResult {
		let new_reserved_stake_amount = self.total_stake_amount.reserved
			.checked_add(&amount)
			.ok_or(ArithmeticError::Overflow)?;
		if new_reserved_stake_amount > self.total_stake_amount.total {
			return Err(ArithmeticError::Overflow.into());
		}
		self.total_stake_amount.reserved = new_reserved_stake_amount;
		Ok(())
	}

	pub fn set_unstake_amount_lock(&mut self, amount: Balance) -> DispatchResult {
		let new_reserved_unstake_amount = self.total_unstake_amount.reserved
			.checked_add(&amount)
			.ok_or(ArithmeticError::Overflow)?;
		if new_reserved_unstake_amount > self.total_unstake_amount.total {
			return Err(ArithmeticError::Overflow.into());
		}
		self.total_unstake_amount.reserved = new_reserved_unstake_amount;
		Ok(())
	}
}