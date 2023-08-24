#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo},
		pallet_prelude::*,
		sp_runtime::traits::{Hash, Zero},
		traits::{Currency, Randomness},
		Blake2_128Concat, Hashable,
	};
	use frame_system::pallet_prelude::{BlockNumberFor, *};
	use scale_info::TypeInfo;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};

	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config:
		frame_system::Config + pallet_insecure_randomness_collective_flip::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The Currency handler for the Kitties pallet.
		type Currency: Currency<Self::AccountId>;

		type KittyRandomness: Randomness<Self::Hash, BlockNumberFor<Self>>;

		/// The maximum amount of kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO Part III
	}

	// Set Gender type in kitty struct
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	// We need this to pass kitty info for genesis configuration
	// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	// Struct for holding kitty information
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16],
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: T::AccountId,
	}

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	/// Keeps track of the number of Kitties in existence.
	pub type CountForKitties<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, [u8; 16], Kitty<T>>;

	#[pallet::storage]
	pub type KittiesOwned<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<[u8; 16], T::MaxKittyOwned>,
		ValueQuery,
	>;

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		// TODO Part III
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO Part III: create_kitty

		// TODO Part III: set_price

		// TODO Part III: transfer

		// TODO Part III: buy_kitty

		// TODO Part III: breed_kitty
	}

	// TODO Parts II: helper function for Kitty struct

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}
		fn gen_dna() -> ([u8; 16], Gender) {
			let random = T::KittyRandomness::random(&b"dnaaf"[..]).0;

			let unique_payload = (
				random,
				frame_system::Pallet::<T>::extrinsic_index().unwrap_or_default(),
				frame_system::Pallet::<T>::block_number(),
			);

			let encoded_payload = unique_payload.encode();
			let hash = (&encoded_payload).blake2_128();
			// Generate Gender
			if hash[0] % 2 == 0 {
				// Males are identified by having a even leading byte
				(hash, Gender::Male)
			} else {
				// Females are identified by having a odd leading byte
				(hash, Gender::Female)
			}
		}
		// TODO Part III: helper functions for dispatchable functions

		// TODO: increment_nonce, random_hash, mint, transfer_from
	}
}
