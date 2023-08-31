#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		dispatch::{DispatchResult, DispatchResultWithPostInfo, Vec},
		pallet_prelude::*,
		sp_runtime,
		sp_runtime::{
			traits::{Hash, Zero},
			ArithmeticError,
		},
		traits::{Currency, ExistenceRequirement, Randomness},
		Blake2_128Concat, Hashable,
	};
	use frame_system::{
		config_preludes,
		pallet_prelude::{BlockNumberFor, OriginFor, *},
		Origin,
	};
	use scale_info::TypeInfo;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[cfg(feature = "std")]
	use serde::{Deserialize, Serialize};

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
		/// A new kitty was successfully created.
		Created { kitty: [u8; 16], owner: T::AccountId },
		/// The price of a kitty was successfully set.
		PriceSet { kitty: [u8; 16], price: Option<BalanceOf<T>> },
		/// A kitty was successfully transferred.
		Transferred { from: T::AccountId, to: T::AccountId, kitty: [u8; 16] },
		/// A kitty was successfully sold.
		Sold { seller: T::AccountId, buyer: T::AccountId, kitty: [u8; 16], price: BalanceOf<T> },
	}

	// Set Gender type in kitty struct
	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	// We need this to pass kitty info for genesis configuration
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16])>,
	}

	// #[cfg(feature = "std")]
	// impl<T: Config> Default for GenesisConfig<T> {
	// 	fn default() -> GenesisConfig<T> {
	// 		GenesisConfig { kitties: vec![] }
	// 	}
	// }

	#[pallet::genesis_build]
	impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the DNA and Gender to be
			// supplied
			// for (account, dna, gender) in &self.kitties {
			// 	assert!(Pallet::<T>::mint(account, *dna, *gender).is_ok());
			// }
		}
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// An account may only own `MaxKittiesOwned` kitties.
		TooManyOwned,
		/// Trying to transfer or buy a kitty from oneself.
		TransferToSelf,
		/// This kitty already exists!
		DuplicateKitty,
		/// This kitty does not exist!
		NoKitty,
		/// You are not the owner of this kitty.
		NotOwner,
		/// This kitty is not for sale.
		NotForSale,
		/// Ensures that the buying price is greater than the asking price.
		BidPriceTooLow,
		/// You need to have two cats with different gender to breed.
		CantBreed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?; //

			// Generate unique DNA and Gender
			let (kitty_gen_dna, gender) = Self::gen_dna();

			let kitty_id = Self::mint(&sender, kitty_gen_dna, gender)?;

			frame_support::log::info!("A kitty is born with ID: {:?}.", kitty_id);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn set_price(
			origin: OriginFor<T>,
			kitty_id: [u8; 16],
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			// Ensure the kitty exists and is called by the kitty owner
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == sender, Error::<T>::NotOwner);

			// Set the price in storage
			kitty.price = new_price;
			Kitties::<T>::insert(&kitty_id, kitty);

			Self::deposit_event(Event::PriceSet { kitty: kitty_id, price: new_price });

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(10_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: [u8; 16],
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;
			let kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == from, Error::<T>::NotOwner);
			Self::do_transfer(kitty_id, to, None)?;
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(10_000)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: [u8; 16],
			limit_price: BalanceOf<T>,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let buyer = ensure_signed(origin)?;
			// Transfer the kitty from seller to buyer as a sale
			Self::do_transfer(kitty_id, buyer, Some(limit_price))?;

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(10_000)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			parent_1: [u8; 16],
			parent_2: [u8; 16],
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let sender = ensure_signed(origin)?;

			let maybe_mom = Kitties::<T>::get(&parent_1).ok_or(Error::<T>::NoKitty)?;
			let maybe_dad = Kitties::<T>::get(&parent_2).ok_or(Error::<T>::NoKitty)?;

			// Check both parents are owned by the caller of this function
			ensure!(maybe_mom.owner == sender, Error::<T>::NotOwner);
			ensure!(maybe_dad.owner == sender, Error::<T>::NotOwner);

			// Parents must be of opposite genders
			ensure!(maybe_mom.gender != maybe_dad.gender, Error::<T>::CantBreed);

			// Create new DNA from these parents
			let (new_dna, new_gender) = Self::breed_dna(&parent_1, &parent_2);

			// Mint new kitty
			Self::mint(&sender, new_dna, new_gender)?;
			Ok(())
		}
	}

	// TODO Parts II: helper function for Kitty struct

	impl<T: Config> Pallet<T> {
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
		pub fn mint(
			owner: &T::AccountId,
			dna: [u8; 16],
			gender: Gender,
		) -> Result<[u8; 16], DispatchError> {
			let kitty = Kitty::<T> { dna, price: None, gender, owner: owner.clone() };

			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);

			let count = CountForKitties::<T>::get();
			let new_count = count.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			// Append kitty to KittiesOwner
			KittiesOwned::<T>::try_append(&owner, kitty.dna)
				.map_err(|_| Error::<T>::TooManyOwned)?;

			// Write new kitty to storage
			Kitties::<T>::insert(kitty.dna, kitty);
			CountForKitties::<T>::put(new_count);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created { kitty: dna, owner: owner.clone() });

			// Returns the DNA of the new kitty if this succeeds
			Ok(dna)
		}

		pub fn do_transfer(
			kitty_id: [u8; 16],
			to: T::AccountId,
			maybe_limit_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let mut kitty = Kitties::<T>::get(&kitty_id).ok_or(Error::<T>::NoKitty)?;
			let from = kitty.owner;

			ensure!(from != to, Error::<T>::TransferToSelf);
			let mut from_owned = KittiesOwned::<T>::get(&from);

			// Remove kitty from list of owned kitties.
			if let Some(ind) = from_owned.iter().position(|&id| id == kitty_id) {
				from_owned.swap_remove(ind);
			} else {
				return Err(Error::<T>::NoKitty.into())
			}

			// Add kitty to the list of owned kitties.
			let mut to_owned = KittiesOwned::<T>::get(&to);
			to_owned.try_push(kitty_id).map_err(|_| Error::<T>::TooManyOwned)?;

			// Mutating state here via a balance transfer, so nothing is allowed to fail after this.
			// The buyer will always be charged the actual price. The limit_price parameter is just
			// a protection so the seller isn't able to front-run the transaction.
			if let Some(limit_price) = maybe_limit_price {
				if let Some(price) = kitty.price {
					ensure!(limit_price >= price, Error::<T>::BidPriceTooLow);
					// Transfer the amount from buyer to seller
					T::Currency::transfer(&to, &from, price, ExistenceRequirement::KeepAlive)?;
					// Deposit sold event
					Self::deposit_event(Event::Sold {
						seller: from.clone(),
						buyer: to.clone(),
						kitty: kitty_id,
						price,
					});
				} else {
					// Kitty price is set to `None` and is not for sale
					return Err(Error::<T>::NotForSale.into())
				}
			}
			// Transfer succeeded, update the kitty owner and reset the price to `None`.
			kitty.owner = to.clone();
			kitty.price = None;

			// Write updates to storage
			Kitties::<T>::insert(&kitty_id, kitty);
			KittiesOwned::<T>::insert(&to, to_owned);
			KittiesOwned::<T>::insert(&from, from_owned);

			Self::deposit_event(Event::Transferred { from, to, kitty: kitty_id });

			Ok(())
		}

		pub fn breed_dna(parent1: &[u8; 16], parent2: &[u8; 16]) -> ([u8; 16], Gender) {
			let (mut new_dna, new_gender) = Self::gen_dna();

			for i in 0..new_dna.len() {
				new_dna[i] = Self::mutate_dna_fragment(parent1[i], parent2[i], new_dna[i])
			}

			(new_dna, new_gender)
		}

		fn mutate_dna_fragment(dna_fragment1: u8, dna_fragment2: u8, random_value: u8) -> u8 {
			if random_value % 2 == 0 {
				// either return `dna_fragment1` if its an even value
				dna_fragment1
			} else {
				// or return `dna_fragment2` if its an odd value
				dna_fragment2
			}
		}
	}
}
