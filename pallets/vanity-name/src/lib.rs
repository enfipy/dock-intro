#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use codec::{Codec, Encode, Decode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch,
    traits::{Currency, Get},
    Parameter,
};
use frame_system::ensure_signed;
use sp_runtime::{RuntimeDebug, traits::{Member, UniqueSaturatedFrom, UniqueSaturatedInto}};

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Trait: pallet_balances::Trait {
    /// Because this pallet emits events, it depends on the runtime's definition of an event.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// String representation.
    type String: From<Vec<u8>> + Into<Vec<u8>> + Codec + Clone + Parameter + Member + Default;

    type MinVanityNameLength: Get<u32>;
    type MaxVanityNameLength: Get<u32>;
    type MaxVanityNamePrice: Get<<Self as pallet_balances::Trait>::Balance>;
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
        pub NameToAccount get(fn name_to_account): map hasher(blake2_128_concat) T::String => Option<T::AccountId>;
        pub AccountToName get(fn account_to_name): map hasher(blake2_128_concat) T::AccountId => Option<u128>;
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

            let amount: u128 = amount.unique_saturated_into();
            let base_price: u128 = T::MaxVanityNamePrice::get().unique_saturated_into();
            let price: u128 = ((base_price as f64) / (sname.len() as f64)).floor() as u128;
            let total_balance = <pallet_balances::Module<T>>::total_balance(&who);
            if total_balance.unique_saturated_into() < price || price > amount {
                Err(Error::<T>::NotEnoughBalance)?;
            }

            let block_number = <frame_system::Module<T>>::block_number().unique_saturated_into();
            let registered_until = (block_number + (amount / price)).unique_saturated_into();
            let vanity_name = VanityName {
                name: name.clone(),
                price: T::Balance::unique_saturated_from(price),
                registered_until,
                owner: who.clone(),
            };

            // TODO: Remove. I think this field is not necessary anymore.
            let registered_names_count = Self::registered_names_count() as u128 + 1;
            <RegisteredNames<T>>::mutate(registered_names_count, |val| *val = Some(vanity_name));
            RegisteredNamesCount::mutate(|val| *val = *val + registered_names_count as u64);
            <AccountToName<T>>::mutate(who.clone(), |val| *val = Some(registered_names_count));
            <NameToAccount<T>>::mutate(name.clone(), |val| *val = Some(who.clone()));

            Self::deposit_event(RawEvent::NameRegistered(who, name));
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        #[weight = 10_000 + T::DbWeight::get().reads_writes(1,1)]
        pub fn renew_registration(origin) -> dispatch::DispatchResult {
            let _who = ensure_signed(origin)?;

            // Read a value from storage.
            match None as Option<u32> {
                // Return an error if the value has not been set.
                None => Err(Error::<T>::NoneValue)?,
                Some(old) => {
                    // Increment the value read from storage; will error in the event of overflow.
                    let _ = old.checked_add(1).ok_or(Error::<T>::NoneValue)?;
                    // Update the value in storage with the incremented result.
                    // Something::put(new);
                    Ok(())
                },
            }
        }
    }
}
