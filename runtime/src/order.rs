use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct Order<TokenBalance, AccountId, AssetId, Hash, Status> {
    id: Hash,
    owner: AccountId,
    btotal: TokenBalance,  // 借款总额
    btoken_id: AssetId,    // 借款币种
    already: TokenBalance, // 已经借到
    duration: u64,         // 借款时长
    stotal: TokenBalance,  // 抵押总额
    stoken_id: AssetId,    // 抵押币种
    interest: u32,         // 年利率，万分之 x
    state: Status,         // 订单状态
}

#[derive(Debug,Encode, Decode, Clone, PartialEq, Eq, Copy)]
pub enum OrderStatus{
    Created,
    PartialFilled,
    Filled,
    Canceled,
    breaked,
}

#[derive(Debug,Encode, Decode, Clone, PartialEq, Eq, Copy)]
pub enum OrderType{
    Borrow,
    Supply,
}

pub trait Trait: system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as SubstrateModuleTemplate {
		Something get(something): Option<u32>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event<T>() = default;

		pub fn do_something(origin, something: u32) -> Result {

			let who = ensure_signed(origin)?;


			<Something<T>>::put(something);


			Self::deposit_event(RawEvent::SomethingStored(something, who));
			Ok(())
		}
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as system::Trait>::AccountId {

		SomethingStored(u32, AccountId),
	}
);