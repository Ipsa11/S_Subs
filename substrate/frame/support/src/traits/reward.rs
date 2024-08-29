use crate::pallet_prelude::DispatchError;
use scale_info::prelude::vec::Vec;
use sp_runtime::DispatchResult;
pub trait Rewards<AccountId>{
	fn payout_validators() -> Vec<AccountId>;
	fn claim_rewards(account:AccountId) -> Result<(), DispatchError>;
	fn calculate_reward() -> DispatchResult;
	fn reward_percent() -> DispatchResult;
}
