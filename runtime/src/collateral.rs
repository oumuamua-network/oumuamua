// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.





use parity_codec::{Decode, Encode};
use rstd::cmp;
use rstd::prelude::*;
// use runtime_primitives::traits::SimpleArithmetic;
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub, Member, SimpleArithmetic, Hash, Zero};

// use runtime_primitives::traits::{As};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ReservableCurrency},
    Parameter, StorageMap, StorageValue,
};
use system::ensure_signed;

use crate::oumuamua;

pub trait Trait: oumuamua::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

}

// type CollateralIndex = u64;
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event() = default;
		
		pub fn add_collateral(
			origin,
			account: T::AccountId,
			amount: T::Balance,
			asset : T::AssetId,
			order_id: T::Hash
		)  -> Result{
			let sender = ensure_signed(origin)?;
			
			Self::deposit_event(RawEvent::Addcollateral(sender, asset, amount));
			Self::do_add_collateral(account, amount, asset, order_id)
		}

		pub fn remove_collateral(origin, account:T::AccountId, hash: T::Hash) ->Result {
			let sender = ensure_signed(origin)?;
			Self::deposit_event(RawEvent::Removecollateral(sender, hash));
			Self::do_remove_collateral(account, hash)
		}
	}
}

#[derive(Debug,Encode, Decode, Clone, PartialEq, Eq, Copy)]
pub enum CollateralStatus{
    Created,
	Hot,
    Canceled,
    Filled,
}

/// A spending collateral.
#[derive(Debug, Encode, Decode, Clone, PartialEq, Eq)]
pub struct Collateral <T> where T: Trait{
	account: T::AccountId,
	amount: T::Balance,
	asset : T::AssetId,
	status: CollateralStatus,
	// timestamp : T::Moment,
	order_id: T::Hash,
	hash : T::Hash,
}


impl<T:Trait> Collateral<T>{
    pub fn new (account:T::AccountId, amount: T::Balance, asset: T::AssetId, order_id: T::Hash) ->Self {
		// use std::time::SystemTime;
		// let now = SystemTime::now();
        // now.duration_since(SystemTime::UNIX_EPOCH)
		// 	.map_err(|_| {
		// 			// return Err("Current time is before unix epoch")
		// 			return None
		// 	}).and_then(|d| {
		// 		let timestamp: T::Moment = d.as_millis() as u64;
				
		// 	})
        let hash = (account.clone(), asset, order_id).using_encoded(<T as system::Trait>::Hashing::hash);
        Collateral {
            account, amount, asset, 
			status: CollateralStatus::Created,
			order_id, hash
        }
        
    }
    fn is_finished(&self) -> bool {
        // (self.amount == Zero::zero() && self.status == CollateralStatus::Filled)|| self.status == CollateralStatus::Canceled
        self.status == CollateralStatus::Filled|| self.status == CollateralStatus::Canceled
    }

}

decl_storage! {
	trait Store for Module<T: Trait> as Collateral {
		/// Number of collaterals that have been made.
		// collateralCount get(collateral_count): CollateralIndex;

		/// collaterals that have been made.
		/// (account, hash) => Collateral
		Collaterals get(collaterals): map (T::AccountId ,T::Hash) => Option<Collateral<T>>;

	}
}

impl<T:Trait>  Collaterals<T>{
}

decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::AccountId,
    	<T as system::Trait>::Hash,
    	<T as oumuamua::Trait>::AssetId,
    	<T as balances::Trait>::Balance,
	{
		/// New collateral.
		CollateralCreate(AccountId, Balance, AssetId, Hash),
		/// collateral remove.
		CollateralRemove(AccountId, Hash),
		
	}
);

impl<T: Trait> Module<T> {
	/// The needed bond for a collateral whose spend is `value`.
	fn do_add_collateral(account: T::AccountId, amount: T::Balance, asset: T::AssetId, order_id: T::Hash) -> Result {
		let new_collateral = Collateral::<T>::new(account, amount, asset, order_id);
		ensure!(new_collateral.is_some(), "Current time is before unix epoch")?;
		let new_collateral = new_collateral.unwrap();
		let hash = new_collateral.hash;
		// ensure!(<Accounts<T>>::exists(account), "account not registered");
		ensure!(!<Collaterals<T>>::exists((account, hash)), "collateral conflicts")?;
		// if <Collaterals<T>>::exists((account, hash)){
		// 	old_collateral = Self.collaterals((account, hash));
		// 	ensure!(old_collateral.is_some(),"old collateral not found") ;
		// 	old_collateral = old_collateral.unwrap();
		// 	new_collateral = new_collateral + old_collateral;
		// }else{
			
		// }
		<Collaterals<T>>::insert((account, hash), new_collateral);
		Self::deposit_event(RawEvent::CollateralCreate(account.clone(), amount, asset, order_id));
		Ok(())
	}
	fn do_cancel_collateral(account: T::AccountId, hash: T::Hash) -> Result{
		ensure!(<Collaterals<T>>::exists((account, hash)), "collateral not found")?;

		let mut target_collateral = Self.collaterals((account, hash)).unwrap();
		let old_status = target_collateral.status;
		ensure!(old_status == CollateralStatus::Created, "status can not turn to hot once it's not created")?;
		// if old_status != CollateralStatus::Created {
		// 	return false
		// }
		target_collateral.status = CollateralStatus::Canceled;
		<Collaterals<T>>::insert((account, hash), target_collateral);
		Self::do_remove_collateral(account, hash)
	}
	fn do_fill_collateral(account: T::AccountId, hash: T::Hash) -> Result{
		ensure!(<Collaterals<T>>::exists((account, hash)), "collateral not found");
		// if !<Collaterals<T>>::exists((account, hash)){
		// 	return false
		// }
		let mut target_collateral = Self.collaterals((account, hash)).unwrap();
		let old_status = target_collateral.status;
		ensure!(old_status == CollateralStatus::Created, "status can not turn to hot once it's not created")?;
		// if old_status != CollateralStatus::Hot {
		// 	return false
		// }
		target_collateral.status = CollateralStatus::Filled;
		<Collaterals<T>>::insert((account, hash), target_collateral);
		Self::do_remove_collateral(account, hash)
		// true
	}
	fn do_make_collateral_hot(account: T::AccountId, hash: T::Hash) -> Result{
		ensure!(<Collaterals<T>>::exists((account, hash)), "collateral not found")?;
		// if !<Collaterals<T>>::exists((account, hash)){
		// 	return false
		// }
		let mut target_collateral = Self.collaterals((account, hash)).unwrap();
		let old_status = target_collateral.status;
		ensure!(old_status == CollateralStatus::Created, "status can not turn to hot once it's not created")?;
		// if old_status != CollateralStatus::Created {
		// 	return false
		// }
		target_collateral.status = CollateralStatus::Hot;
		<Collaterals<T>>::insert((account, hash), target_collateral);
		// true
		Ok(())
	}

	fn do_remove_collateral(account: T::AccountId, hash: T::Hash) -> Result {
		ensure!(<Collaterals<T>>::exists((account, hash)), "collateral not found")?;
		let target_collateral = Self.collaterals((account, hash)).unwrap();
		if target_collateral.is_finished(){
			<Collaterals<T>>::remove((account, hash));
		}else{
			return Err("collateral not finished")
		}
		<Collaterals<T>>::remove((account, hash));
		Self::deposit_event(RawEvent::CollateralRemove(account,hash));
		Ok(())

	}
}
