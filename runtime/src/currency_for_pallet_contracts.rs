//! Only the `<CurrencyForPalletContracts as Currency>::transfer()` function must be different than the `<Balances as Currency>::transfer()` function.
//! Everything else MUST BE THE SAME!

use frame_support::{
	traits::{Currency, ReservableCurrency, WithdrawReasons, ExistenceRequirement, SignedImbalance, BalanceStatus},
	dispatch::{DispatchError, DispatchResult},
};

use super::{AccountId, Balances};

pub struct CurrencyForPalletContracts;

impl Currency<AccountId> for CurrencyForPalletContracts {

	type Balance = <Balances as Currency<AccountId>>::Balance;
	type PositiveImbalance = <Balances as Currency<AccountId>>::PositiveImbalance;
	type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

	fn total_balance(who: &AccountId) -> Self::Balance {
		<Balances as Currency<AccountId>>::total_balance(who)
	}
	fn can_slash(who: &AccountId, value: Self::Balance) -> bool {
		<Balances as Currency<AccountId>>::can_slash(who, value)
	}
	fn total_issuance() -> Self::Balance {
		<Balances as Currency<AccountId>>::total_issuance()
	}
	fn minimum_balance() -> Self::Balance {
		<Balances as Currency<AccountId>>::minimum_balance()
	}
	fn burn(amount: Self::Balance) -> Self::PositiveImbalance {
		<Balances as Currency<AccountId>>::burn(amount)
	}
	fn issue(amount: Self::Balance) -> Self::NegativeImbalance {
		<Balances as Currency<AccountId>>::issue(amount)
	}
	fn pair(amount: Self::Balance) -> (Self::PositiveImbalance, Self::NegativeImbalance) {
		<Balances as Currency<AccountId>>::pair(amount)
	}
	fn free_balance(who: &AccountId) -> Self::Balance {
		<Balances as Currency<AccountId>>::free_balance(who)
	}
	fn ensure_can_withdraw(
		who: &AccountId,
		_amount: Self::Balance,
		reasons: WithdrawReasons,
		new_balance: Self::Balance,
	) -> DispatchResult {
		<Balances as Currency<AccountId>>::ensure_can_withdraw(who, _amount, reasons, new_balance)
	}
	fn transfer(
		source: &AccountId,
		dest: &AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		// <Balances as Currency<AccountId>>::transfer(source, dest, value, existence_requirement)
		Balances::do_transfer(source, dest, value, existence_requirement)?;
		Ok(())
	}
	fn slash(who: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		<Balances as Currency<AccountId>>::slash(who, value)
	}
	fn deposit_into_existing(
		who: &AccountId,
		value: Self::Balance,
	) -> Result<Self::PositiveImbalance, DispatchError> {
		<Balances as Currency<AccountId>>::deposit_into_existing(who, value)
	}
	fn resolve_into_existing(
		who: &AccountId,
		value: Self::NegativeImbalance,
	) -> Result<(), Self::NegativeImbalance> {
		<Balances as Currency<AccountId>>::resolve_into_existing(who, value)
	}
	fn deposit_creating(who: &AccountId, value: Self::Balance) -> Self::PositiveImbalance {
		<Balances as Currency<AccountId>>::deposit_creating(who, value)
	}
	fn resolve_creating(who: &AccountId, value: Self::NegativeImbalance) {
		<Balances as Currency<AccountId>>::resolve_creating(who, value)
	}
	fn withdraw(
		who: &AccountId,
		value: Self::Balance,
		reasons: WithdrawReasons,
		liveness: ExistenceRequirement,
	) -> Result<Self::NegativeImbalance, DispatchError> {
		<Balances as Currency<AccountId>>::withdraw(who, value, reasons, liveness)
	}
	fn settle(
		who: &AccountId,
		value: Self::PositiveImbalance,
		reasons: WithdrawReasons,
		liveness: ExistenceRequirement,
	) -> Result<(), Self::PositiveImbalance> {
		<Balances as Currency<AccountId>>::settle(who, value, reasons, liveness)
	}
	fn make_free_balance_be(
		who: &AccountId,
		balance: Self::Balance,
	) -> SignedImbalance<Self::Balance, Self::PositiveImbalance> {
		<Balances as Currency<AccountId>>::make_free_balance_be(who, balance)
	}
}
impl ReservableCurrency<AccountId> for CurrencyForPalletContracts {
	fn can_reserve(who: &AccountId, value: Self::Balance) -> bool {
		<Balances as ReservableCurrency<AccountId>>::can_reserve(who, value)
	}
	fn slash_reserved(
		who: &AccountId,
		value: Self::Balance,
	) -> (Self::NegativeImbalance, Self::Balance) {
		<Balances as ReservableCurrency<AccountId>>::slash_reserved(who, value)
	}
	fn reserved_balance(who: &AccountId) -> Self::Balance {
		<Balances as ReservableCurrency<AccountId>>::reserved_balance(who)
	}
	fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult {
		<Balances as ReservableCurrency<AccountId>>::reserve(who, value)
	}
	fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance {
		<Balances as ReservableCurrency<AccountId>>::unreserve(who, value)
	}
	fn repatriate_reserved(
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> Result<Self::Balance, DispatchError> {
		<Balances as ReservableCurrency<AccountId>>::repatriate_reserved(slashed, beneficiary, value, status)
	}
}
