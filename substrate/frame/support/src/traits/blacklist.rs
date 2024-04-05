use scale_info::prelude::vec::Vec;

pub trait BlackListAccounts<AccountId> {
    fn blacklisted_accounts() -> Vec<AccountId>;
}