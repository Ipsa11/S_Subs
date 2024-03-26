use scale_info::prelude::vec::Vec;
use crate::pallet_prelude::DispatchResult;
pub trait DerivativeRewardAccount<AccountId>{
	fn derivative_reward_accounts() -> Vec<AccountId>;
	fn claim_derivative(account:AccountId) -> DispatchResult;
	fn reset_reward() -> DispatchResult;
}

pub trait StakingAccount<AccountId> {
	fn staking_account() -> AccountId;
}
