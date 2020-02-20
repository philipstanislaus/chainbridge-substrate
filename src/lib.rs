#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, traits::Currency,
    traits::ExistenceRequirement::AllowDeath,
};
use frame_system::{self as system, ensure_signed};
use sp_std::vec::Vec;

mod mock;
mod tests;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The currency mechanism.
    type Currency: Currency<Self::AccountId>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Counter(u32);

/// Tracks deposit count for an associated chain
impl Counter {
    fn increment(&mut self) {
        self.0 += 1;
    }
}

decl_event!(
    pub enum Event<T> where <T as frame_system::Trait>::Hash {
        // dest_id, deposit_id, to, token_id, metadata
        AssetTransfer(Vec<u8>, u32, Vec<u8>, Vec<u8>, Vec<u8>),
        UselessEvent(Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        // Interactions with this chain is not permitted
        ChainNotWhitelisted
    }
}

decl_storage!(
    trait Store for Module<T: Trait> as Bridge {
        EmitterAddress: Vec<u8>;

        Chains: map
            hasher(blake2_256) Vec<u8>
            => Option<Counter>;

        // See https://github.com/paritytech/substrate/blob/5135844eb77eb5dd76948a535146c0ad1df6bd0f/frame/balances/src/lib.rs#L373
        // And https://github.com/paritytech/substrate/blob/ddb309ae7c70e5e51a60879af18819cf28be4a32/frame/indices/src/lib.rs#L93
        EndowedAccount get(fn endowed) config(): T::AccountId;
        // add_extra_genesis {
        //     config(endowed_account): T::AccountId;
        // }

    }
);

decl_module!(
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Default method for emitting events
        fn deposit_event() = default;

        /// Sets the address used to identify this chain
        pub fn set_address(origin, addr: Vec<u8>) -> DispatchResult {
            // TODO: Limit access
            ensure_signed(origin)?;
            EmitterAddress::put(addr);
            Ok(())
        }

        /// Enables a chain ID as a destination for a bridge transfer
        pub fn whitelist_chain(origin, id: Vec<u8>) -> DispatchResult {
            // TODO: Limit access
            ensure_signed(origin)?;
            Chains::insert(&id, Counter(0));
            Ok(())
        }

        /// Commits an asset transfer to the chain as an event to be acted on by the bridge.
        pub fn transfer_asset(origin, dest_id: Vec<u8>, to: Vec<u8>, token_id: Vec<u8>, metadata: Vec<u8>) -> DispatchResult {
            // TODO: Limit access
            ensure_signed(origin)?;
            // Ensure chain is whitelisted
            if let Some(mut counter) = Chains::get(&dest_id) {
                // Increment counter and store
                counter.increment();
                Chains::insert(&dest_id, counter.clone());
                Self::deposit_event(RawEvent::AssetTransfer(dest_id, counter.0, to, token_id, metadata));
                Ok(())
            } else {
                Err(Error::<T>::ChainNotWhitelisted)?
            }
        }

        // TODO: Should use correct amount type
        pub fn transfer(origin, to: T::AccountId, amount: u32) -> DispatchResult {
            ensure_signed(origin)?;
            let source: T::AccountId = <EndowedAccount<T>>::get();
            T::Currency::transfer(&source, &to, amount.into(), AllowDeath)?;
            Ok(())
        }
    }
);