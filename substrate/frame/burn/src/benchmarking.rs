#![cfg(feature = "runtime-benchmarks")]
use super::*;
use frame_benchmarking::impl_benchmark_test_suite;
use frame_benchmarking::v2::benchmarks;
use frame_system::RawOrigin;
use frame_system::Config as SystemConfig;
pub type DepositBalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as SystemConfig>::AccountId>>::Balance;
use sp_runtime::traits:: Bounded;
use scale_info::prelude::vec;

fn assert_last_event<T: Config>(generic_event: <T as Config>::RuntimeEvent) {
	let events = frame_system::Pallet::<T>::events();
	let system_event: <T as frame_system::Config>::RuntimeEvent = generic_event.into();
	let frame_system::EventRecord { event, .. } = &events[events.len() - 1];
	assert_eq!(event, &system_event);

}


#[allow(unused)]
use crate::Pallet as Burn;
#[benchmarks]
mod benchmarks {
	use super::*;
    #[benchmark]
	fn burn_amount() {
        let _ = T::Currency::make_free_balance_be(&crate::Pallet::<T>::burn_account().unwrap(),DepositBalanceOf::<T>::max_value());
        let value = 100u32.into();
        #[extrinsic_call]
        burn_amount(RawOrigin::Root,value);

        assert_last_event::<T>(Event::Burned{amount:value}.into());
    }


impl_benchmark_test_suite!(Burn, crate::mock::new_test_ext(), crate::test::Test);
}