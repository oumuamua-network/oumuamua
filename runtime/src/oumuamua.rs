use parity_codec::{Decode, Encode};
use rstd::cmp;
use rstd::prelude::*;
use runtime_primitives::traits::SimpleArithmetic;
use runtime_primitives::traits::{As, Hash, Zero};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ReservableCurrency},
    Parameter, StorageMap, StorageValue,
};
use system::ensure_signed;
//use assets::*;
//use  treasury::*;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BorrowOrder<Balance, AccountId, AssetId, Hash> {
    id: Hash,
    owner: AccountId,
    btotal: Balance,    // 借款总额
    btoken_id: AssetId, // 借款币种
    already: Balance,   // 已经借到
    duration: u64,      // 借款时长
    stotal: Balance,    // 抵押总额
    stoken_id: AssetId, // 抵押币种
    interest: u32,      // 年利率，万分之 x
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SupplyOrder<Balance, AccountId, AssetId, Hash> {
    id: Hash,
    owner: AccountId,
    total: Balance,
    stoken: AssetId,      // 提供的资金种类（默认是 USDT）
    tokens: Vec<AssetId>, // 接受抵押的资金种类
    amortgage: u32,       // 接受抵押率，万分之 x
    duration: u64,        // 这部分资金的 free time
    interest: u32,        // 接受最小的年利率，万分之 x
}


pub trait Trait: balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type AssetId: Parameter + SimpleArithmetic + Default + Copy;
}


decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as balances::Trait>::Balance,
        <T as system::Trait>::BlockNumber

    {
        Created(AccountId, Hash),
        PriceSet(AccountId, Hash, Balance),
        Transferred(AccountId, AccountId, Hash),
        Bought(AccountId, AccountId, Hash, Balance),
        AuctionCreated(Hash, Balance, BlockNumber),
        Bid(Hash, Balance, AccountId),
        AuctionFinalized(Hash, Balance, BlockNumber),

        CreateBorrow(AccountId, Balance,  u64, Balance, u32),
        CancelBorrow(AccountId, Hash),
        TakeBorrow(AccountId),
        CreateSupply(AccountId),
        CancelSupply(AccountId, Hash),
        TakeSupply(AccountId),
            
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        BorrowOrderDetail get(borrow_order_detail): map T::Hash => BorrowOrder<T::Balance, T::AccountId, T::AssetId, T::Hash>;
        BorrowOrderOwner get(owner_of_borrow): map T::Hash => Option<T::AccountId>;

        AllBorrowOrder get(borrow_by_index): map u64 => T::Hash;
        AllBorrowOrderCount get(borrow_order_count): u64;
        AllBorrowOrderIndex: map T::Hash => u64;

        OwnedBorrowOrder get(borrow_of_owner_by_index): map(T::AccountId, u64) => T::Hash;
        OwnedBorrowCount get(owned_borrow_count): map T::AccountId => u64;
        OwnedBorrowIndex: map T::Hash => u64;

        SupplyOrderDetail get(supply_order_detail): map T::Hash => SupplyOrder<T::Balance, T::AccountId, T::AssetId, T::Hash>;
        SupplyOrderOwner get(owner_of_supply): map T::Hash => Option<T::AccountId>;

        AllSupplyOrder get(supply_by_index): map u64 => T::Hash;
        AllSupplyOrderCount get(supply_order_count): u64;
        AllSupplyOrderIndex: map T::Hash => u64;

        OwnedSupplyOrder get(supply_of_owner_by_index): map(T::AccountId, u64) => T::Hash;
        OwnedSupplyCount get(owned_supply_count): map T::AccountId => u64;
        OwnedSupplyIndex: map T::Hash => u64;

        AllowAssets get(allow_asset): Vec<T::AssetId>;

        Nonce: u64;       
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        fn create_borrow(origin, btotal: T::Balance, btokenid: T::AssetId, duration: u64, stotal: T::Balance,
                         stokenid: T::AssetId, interest: u32) -> Result {
            let sender = ensure_signed(origin)?;


            
            Ok(())
        }
         

        fn cancel_borrow(orderid: T::Hash) ->Result {


            Ok(())

        }


        


        
    }
}

impl<T: Trait> Module<T> {
    
  
}
