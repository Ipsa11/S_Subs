use crate::pallet_prelude::DispatchError;
use scale_info::prelude::vec::Vec;
pub trait RewardAvailable<Balance>{
	fn reward_available() -> Balance;
}

pub trait RewardAccount<AccountId>{
	fn reward_account() -> Vec<AccountId>;
	fn claim_rewards(account:AccountId) -> Result<(), DispatchError>;
}
