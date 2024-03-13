use frame_support::{ assert_noop, assert_ok, dispatch::RawOrigin, traits::Currency };
use sp_runtime::traits::BadOrigin;
pub use crate::mock::*;
use super::*;


fn set_balance(){
	let _ = BalanceBurn::deposit_creating(&Burn::burn_account().unwrap(),1_000_000_000_000_000);
}

#[test]
fn burn_amount_works() {
	// Initialize the test environment
	new_test_ext().execute_with(|| {
		set_balance();
		let burn_amount = 1000000000000;
		let result = Burn::burn_amount(RawOrigin::Root.into(), burn_amount);
		assert_ok!(result);
	});
}


#[test]
fn burn_fails() {
	// Initialize the test environment
	new_test_ext().execute_with(|| {
		set_balance();
		let burn_amount = 9_000_000_000_000_000;
		let result = Burn::burn_amount(RawOrigin::Root.into(), burn_amount);
		assert_noop!(result,Error::<Test>::InsufficientBalance);
	});
}

#[test]
fn burn_amount_fails_for_non_root() {
	new_test_ext().execute_with(|| {
		let burn_amount = 10;
		let result = Burn::burn_amount(RawOrigin::Signed(1).into(), burn_amount);
		assert_noop!(result, BadOrigin);
	});
}
