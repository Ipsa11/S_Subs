use crate::pallet_prelude::DispatchError;
use scale_info::prelude::vec::Vec;
use sp_runtime::DispatchResult;
pub trait RewardAvailable<Balance>{
	fn reward_available() -> Balance;
}

pub trait Rewards<AccountId>{
	fn reward_account() -> Vec<AccountId>;
	fn claim_rewards(account:AccountId) -> Result<(), DispatchError>;
	fn calculate_reward() -> DispatchResult;
}
