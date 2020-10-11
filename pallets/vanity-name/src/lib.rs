#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Codec, Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Currency, Get, WithdrawReason, LockableCurrency},
    Parameter,
};
use frame_system::ensure_signed;
use sp_runtime::{
    traits::{Member, UniqueSaturatedFrom, UniqueSaturatedInto},
    RuntimeDebug,
};

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: pallet_balances::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// String representation.
    type String: From<Vec<u8>> + Into<Vec<u8>> + Codec + Clone + Parameter + Member + Default;

    type MinVanityNameLength: Get<u32>;
    type MaxVanityNameLength: Get<u32>;
    type MaxVanityNamePrice: Get<<Self as pallet_balances::Trait>::Balance>;
    type MinPeriodToRegister: Get<u128>;
}

#[derive(Clone, Default, PartialEq, RuntimeDebug, Encode, Decode)]
pub struct VanityName<AccountId, Balance, String, BlockNumber> {
    name: String,
    price: Balance,
    registered_until: BlockNumber,
    owner: AccountId,
}

decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        pub RegisteredNamesCount get(fn registered_names_count): u64;
        pub RegisteredNames get(fn registered_names): map hasher(blake2_128_concat) u128 => Option<VanityName<T::AccountId, T::Balance, T::String, T::BlockNumber>>;
        pub NameToNameId get(fn name_to_name_id): map hasher(blake2_128_concat) T::String => Option<u128>;
        pub NameToAccount get(fn name_to_account): map hasher(blake2_128_concat) T::String => Option<T::AccountId>;
        pub AccountToNames get(fn account_to_name): map hasher(blake2_128_concat) T::AccountId => Vec<u128>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Trait>::AccountId,
        String = <T as Trait>::String,
        BlockNumber = <T as frame_system::Trait>::BlockNumber,
    {
        NameRegistered(AccountId, String),
        NameExpired(AccountId, String),
        NameRenewed(AccountId, String, BlockNumber),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Error names should be descriptive.
        NoneValue,
        /// Invalid name.
        InvalidName,
        /// Name already registered.
        NameAlreadyRegistered,
        /// Invalid price.
        InvalidPrice,
        /// Not enough balance
        NotEnoughBalance,
        /// Name registered with different account
        NameRegisteredWithDifferentAccount,
        /// Name not registered
        NameNotRegistered,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;
        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// The minimum length of vanity name.
        const MinVanityNameLength: u32 = T::MinVanityNameLength::get();
        /// The maximum length of vanity name.
        const MaxVanityNameLength: u32 = T::MaxVanityNameLength::get();
        /// The maximum price of vanity name.
        const MaxVanityNamePrice: T::Balance = T::MaxVanityNamePrice::get();
        /// The minimum period to register vanity name in blocks.
        const MinPeriodToRegister: u128 = T::MinPeriodToRegister::get();

        fn on_initialize(now: T::BlockNumber) -> frame_support::weights::Weight {
            // FIXME: Should be done through map with tuples (block, vec[name_ids]) that will be
            // removed on specific block number.
            let mut registered_names_count = Self::registered_names_count() as u128;
            let mut id = 1;
            while id <= registered_names_count {
                if let Some(name) = Self::registered_names(id) {
                    if name.registered_until <= now {
                        <RegisteredNames<T>>::swap(id, registered_names_count);
                        <RegisteredNames<T>>::remove(id);
                        <AccountToNames<T>>::mutate(name.owner.clone(), |val| {
                            if let Some(i) = val.iter().position(|x| *x == registered_names_count) {
                                val.remove(i);
                            }
                        });
                        <NameToAccount<T>>::mutate(name.name.clone(), |val| *val = None);
                        <NameToNameId<T>>::mutate(name.name.clone(), |val| *val = None);
                        registered_names_count -= 1;
                    }
                } else {
                    id += 1;
                }
            }
            RegisteredNamesCount::mutate(|val| *val = registered_names_count as u64);
            return 0;
        }

        #[weight = 10_000 + T::DbWeight::get().writes(1)]
        pub fn register_name(origin, name: T::String, amount: T::Balance) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            let tmp_name: Vec<u8> = name.clone().into();
            let sname: &str = sp_std::str::from_utf8(&tmp_name[..]).map_err(|_| Error::<T>::InvalidName)?;
            if (sname.len() as u32) < T::MinVanityNameLength::get() || (sname.len() as u32) > T::MaxVanityNameLength::get() {
                Err(Error::<T>::InvalidName)?;
            }

            let existed_name = Self::name_to_account(name.clone());
            if existed_name.is_some() {
                Err(Error::<T>::NameAlreadyRegistered)?;
            }

            let s_amount: u128 = amount.unique_saturated_into();
            let base_price: u128 = T::MaxVanityNamePrice::get().unique_saturated_into();
            let price: u128 = ((base_price as f64) / (sname.len() as f64)).floor() as u128;
            let total_balance = <pallet_balances::Module<T>>::total_balance(&who);
            if total_balance.unique_saturated_into() < price || price > s_amount {
                Err(Error::<T>::NotEnoughBalance)?;
            }

            let block_number = <frame_system::Module<T>>::block_number().unique_saturated_into();
            let min_period = T::MinPeriodToRegister::get();
            let registered_until = (block_number + (s_amount / price * min_period)).unique_saturated_into();
            let vanity_name = VanityName {
                name: name.clone(),
                price: T::Balance::unique_saturated_from(price),
                registered_until,
                owner: who.clone(),
            };

            // TODO: Support multiple names.
            const STAKING_ID: [u8; 8] = *b"name_buy";
            <pallet_balances::Module<T>>::set_lock(STAKING_ID, &who, amount, WithdrawReason::Reserve.into());

            // TODO: Remove. I think this field is not necessary anymore.
            let registered_names_count = Self::registered_names_count() as u128 + 1;
            <RegisteredNames<T>>::mutate(registered_names_count, |val| *val = Some(vanity_name));
            RegisteredNamesCount::mutate(|val| *val = *val + registered_names_count as u64);
            <AccountToNames<T>>::mutate(who.clone(), |val| val.push(registered_names_count));
            <NameToAccount<T>>::mutate(name.clone(), |val| *val = Some(who.clone()));
            <NameToNameId<T>>::mutate(name.clone(), |val| *val = Some(registered_names_count));

            Self::deposit_event(RawEvent::NameRegistered(who, name));
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn renew_registration(origin, name: T::String, amount: T::Balance) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            let tmp_name: Vec<u8> = name.clone().into();
            let sname: &str = sp_std::str::from_utf8(&tmp_name[..]).map_err(|_| Error::<T>::InvalidName)?;
            if (sname.len() as u32) < T::MinVanityNameLength::get() || (sname.len() as u32) > T::MaxVanityNameLength::get() {
                Err(Error::<T>::InvalidName)?;
            }

            let existed_name = Self::name_to_account(name.clone()).ok_or(Error::<T>::NameNotRegistered)?;
            if existed_name != who {
                Err(Error::<T>::NameRegisteredWithDifferentAccount)?;
            }

            let s_amount: u128 = amount.unique_saturated_into();
            let base_price: u128 = T::MaxVanityNamePrice::get().unique_saturated_into();
            let price: u128 = ((base_price as f64) / (sname.len() as f64)).floor() as u128;
            let total_balance = <pallet_balances::Module<T>>::total_balance(&who);
            if total_balance.unique_saturated_into() < price || price > s_amount {
                Err(Error::<T>::NotEnoughBalance)?;
            }

            let block_number = <frame_system::Module<T>>::block_number().unique_saturated_into();
            let name_id = Self::name_to_name_id(name.clone()).ok_or(Error::<T>::NameNotRegistered)?;
            let min_period = T::MinPeriodToRegister::get();
            let add_period = s_amount / price * min_period;

            // TODO: Support multiple names.
            const STAKING_ID: [u8; 8] = *b"name_buy";
            <pallet_balances::Module<T>>::extend_lock(STAKING_ID, &who, amount, WithdrawReason::Reserve.into());

            <RegisteredNames<T>>::try_mutate_exists(name_id, |maybe_name| -> Result<(), Error<T>> {
                let mut vanity_name = maybe_name.clone().ok_or(Error::<T>::NameNotRegistered)?;
                vanity_name.registered_until = (vanity_name.registered_until.unique_saturated_into() + add_period).unique_saturated_into();
                *maybe_name = Some(vanity_name);
                Ok(())
            })?;

            Self::deposit_event(RawEvent::NameRenewed(who, name, block_number.unique_saturated_into()));
            Ok(())
        }
    }
}
