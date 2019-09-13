use parity_codec::{Codec,Decode, Encode};
use rstd::cmp;
use rstd::prelude::*;
use runtime_primitives::traits::{As, Hash, CheckedAdd, CheckedSub, Member, SimpleArithmetic, Zero};
use support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, ReservableCurrency},
    Parameter, StorageMap, StorageValue,
};
use system::{self, ensure_signed};
use runtime_primitives::traits::One;

const yiwan: u64 = 10000;


#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct BorrowOrder<TokenBalance, AccountId, AssetId, Hash> {
    id: Hash,
    owner: AccountId,
    btotal: TokenBalance,    // 借款总额
    btoken_id: AssetId, // 借款币种
    already: TokenBalance,   // 已经借到
    duration: u64,      // 借款时长
    stotal: TokenBalance,    // 抵押总额
    stoken_id: AssetId, // 抵押币种
    interest: u32,      // 年利率，万分之 x
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct SupplyOrder<TokenBalance, AccountId, AssetId, Hash> {
    id: Hash,
    owner: AccountId,
    total: TokenBalance,
    stoken: AssetId,      // 提供的资金种类（默认是 USDT）
    tokens: Vec<AssetId>, // 接受抵押的资金种类
    amortgage: u32,       // 接受抵押率，万分之 x
    duration: u64,        // 这部分资金的 free time
    interest: u32,        // 接受最小的年利率，万分之 x
}


#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<U> {
    name: Vec<u8>,
    ticker: Vec<u8>,
    total_supply: U,
}

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type AssetId: Parameter + SimpleArithmetic + Default + Copy;
    type TokenBalance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + As<usize>
        + As<u64>
        + From<u64>;
}


decl_event!(
    pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
        <T as self::Trait>::TokenBalance,
        <T as self::Trait>::AssetId,
    {
        CreateBorrow(AccountId, TokenBalance,  u64, TokenBalance, u32),
        CancelBorrow(AccountId, Hash),
        TakeBorrow(AccountId),
        CreateSupply(AccountId),
        CancelSupply(AccountId, Hash),
        TakeSupply(AccountId),

        Transfer(AssetId, AccountId, AccountId, TokenBalance),
        Approval(AssetId, AccountId, AccountId, TokenBalance),

        Reserve(AssetId, AccountId, TokenBalance),
        UnReserve(AssetId, AccountId, TokenBalance),

    }
);

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        BorrowOrderDetail get(borrow_order_detail): map T::Hash => BorrowOrder<T::TokenBalance, T::AccountId, T::AssetId, T::Hash>;
        BorrowOrderOwner get(owner_of_borrow): map T::Hash => Option<T::AccountId>;

        AllBorrowOrder get(borrow_by_index): map u64 => T::Hash;
        AllBorrowOrderCount get(borrow_order_count): u64;
        AllBorrowOrderIndex: map T::Hash => u64;

        OwnedBorrowOrder get(borrow_of_owner_by_index): map(T::AccountId, u64) => T::Hash;
        OwnedBorrowCount get(owned_borrow_count): map T::AccountId => u64;
        OwnedBorrowIndex: map T::Hash => u64;

        SupplyOrderDetail get(supply_order_detail): map T::Hash => SupplyOrder<T::TokenBalance, T::AccountId, T::AssetId, T::Hash>;
        SupplyOrderOwner get(owner_of_supply): map T::Hash => Option<T::AccountId>;

        AllSupplyOrder get(supply_by_index): map u64 => T::Hash;
        AllSupplyOrderCount get(supply_order_count): u64;
        AllSupplyOrderIndex: map T::Hash => u64;

        OwnedSupplyOrder get(supply_of_owner_by_index): map(T::AccountId, u64) => T::Hash;
        OwnedSupplyCount get(owned_supply_count): map T::AccountId => u64;
        OwnedSupplyIndex: map T::Hash => u64;

        AllowAssets get(allow_asset): Vec<T::AssetId>;

        Nonce: u64;


        TokenId get(token_id) config(): T::AssetId;
        Tokens get(token_details): map T::AssetId => Erc20Token<T::TokenBalance>;
        BalanceOf get(balance_of): map (T::AssetId, T::AccountId) => T::TokenBalance;
        FreeBalanceOf get(free_balance_of): map (T::AssetId, T::AccountId) => T::TokenBalance;
        ReserveBalanceOf get(reserve_balance_of): map(T::AssetId, T::AccountId) => T::TokenBalance;

        Allowance get(allowance): map (T::AssetId, T::AccountId, T::AccountId) => T::TokenBalance;

        Admin get(admin) config(): T::AccountId;

// 本来这里应该是通过一个 Oracle 来获取价格。为了简便，直接用数据保存了。初始化时，设置 token_id 为 1 的是 USDT，
// TokenPrice 表示 每单元该币种 能兑换 TokenPrice/10000 的 USDT。
        TokenPrice get(token_price): map T::AssetId => u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        fn init(origin, name: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(sender == Self::admin(), "only Admin can new a token");

            ensure!(name.len() <= 64, "token name cannot exceed 64 bytes");
            ensure!(ticker.len() <= 32, "token ticker cannot exceed 32 bytes");

            let token_id = Self::token_id();

            <TokenId<T>>::mutate(|id| *id += One::one());

            let token = Erc20Token {
                name,
                ticker,
                total_supply,
            };

            <Tokens<T>>::insert(token_id, token);
            <BalanceOf<T>>::insert((token_id, sender.clone()), total_supply);
            <FreeBalanceOf<T>>::insert((token_id, sender.clone()), total_supply);
            <ReserveBalanceOf<T>>::insert((token_id, sender.clone()), T::TokenBalance::from(0u64));

            Ok(())
        }

        fn set_price(origin, token_id: T::AssetId, price: u64) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(sender == Self::admin(), "only Admin can set a token price");

            ensure!(<Tokens<T>>::exists(token_id), "the token does not exist");

            <TokenPrice<T>>::insert(token_id, price);

            Ok(())
        }


        fn transfer(_origin, token_id: T::AssetId, to: T::AccountId, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(_origin)?;
            Self::_transfer(token_id, sender, to, value)
        }

        fn approve(_origin, token_id: T::AssetId, spender: T::AccountId, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(_origin)?;
            ensure!(<BalanceOf<T>>::exists((token_id, sender.clone())), "Account does not own this token");

            let allowance = Self::allowance((token_id, sender.clone(), spender.clone()));
            let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((token_id, sender.clone(), spender.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(token_id, sender.clone(), spender.clone(), value));

            Ok(())
        }

      // the ERC20 standard transfer_from function
      // implemented in the open-zeppelin way - increase/decrease allownace
      // if approved, transfer from an account to another account without owner's signature
        pub fn transfer_from(_origin, token_id: T::AssetId, from: T::AccountId, to: T::AccountId, value: T::TokenBalance) -> Result {
            ensure!(<Allowance<T>>::exists((token_id, from.clone(), to.clone())), "Allowance does not exist.");
            let allowance = Self::allowance((token_id, from.clone(), to.clone()));
            ensure!(allowance >= value, "Not enough allowance.");

            // using checked_sub (safe math) to avoid overflow
            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((token_id, from.clone(), to.clone()), updated_allowance);

            Self::deposit_event(RawEvent::Approval(token_id, from.clone(), to.clone(), value));
            Self::_transfer(token_id, from, to, value)
        }


        fn create_borrow(origin, btotal: T::TokenBalance, btokenid: T::AssetId, duration: u64, stotal: T::TokenBalance,
                         stokenid: T::AssetId, interest: u32) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(
                <BalanceOf<T>>::exists((stokenid, sender.clone())),
                "Account does not own this token"
            );

            ensure!(<Tokens<T>>::exists(btokenid), "the btoken does not exist");

            ensure!(<TokenPrice<T>>::exists(btokenid), "the btoken price does not exist");
            ensure!(<TokenPrice<T>>::exists(stokenid), "the stoken price does not exist");

            let bprice = Self::token_price(btokenid);
            // todo: need checked_mul
            let btotalprice = btotal * T::TokenBalance::from(bprice);

            let sprice = Self::token_price(stokenid);
            // todo: need checked_mul
            let stotalprice = stotal * T::TokenBalance::from(sprice);

            ensure!(stotalprice >= btotalprice, "the value of supply lower than borrow"); // 等额或超额抵押，还没考虑手续费。

            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            Self::_reserve(stokenid, sender.clone(), stotal);

            ensure!(!<BorrowOrderOwner<T>>::exists(random_hash), "Borrow order already exists");

            let new_borrow_order = BorrowOrder {
                id: random_hash,
                owner: sender.clone(),
                btotal: btotal,
                btoken_id: btokenid,
                already: T::TokenBalance::from(0u64),
                duration: duration,
                stotal: stotal,
                stoken_id: stokenid,
                interest: interest,
            };

            let owned_borrow_count = Self::owned_borrow_count(&sender);
            let new_owned_borrow_count = owned_borrow_count.checked_add(1)
                .ok_or("Overflow add a new borrow order to acount")?;

            let all_borrow_order_count = Self::borrow_order_count();
            let new_all_borrow_order_count = all_borrow_order_count.checked_add(1)
                .ok_or("Overflow adding a new borrow order")?;

            <BorrowOrderDetail<T>>::insert(random_hash, new_borrow_order);
            <BorrowOrderOwner<T>>::insert(random_hash, &sender);

            <AllBorrowOrder<T>>::insert(all_borrow_order_count, random_hash);
            <AllBorrowOrderCount<T>>::put(new_all_borrow_order_count);
            <AllBorrowOrderIndex<T>>::insert(random_hash, all_borrow_order_count);

            <OwnedBorrowOrder<T>>::insert((sender.clone(), owned_borrow_count), random_hash);
            <OwnedBorrowCount<T>>::insert(&sender, new_owned_borrow_count);
            <OwnedBorrowIndex<T>>::insert(random_hash, owned_borrow_count);


            <Nonce<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::CreateBorrow(sender, btotal, duration, stotal, interest));

            Ok(())
        }


        fn cancel_borrow(orderid: T::Hash) ->Result {


            Ok(())

        }

        fn take_borrow(origin) -> Result {

            Ok(())
        }


        fn create_Supply(origin, stotal: T::TokenBalance, stokenid: T::AssetId, btokenids: Vec<T::AssetId>, amortgage: u32,
                           duration: u64, interest: u32) -> Result {
            let sender = ensure_signed(origin)?;

            ensure!(
                <BalanceOf<T>>::exists((stokenid, sender.clone())),
                "Account does not own this token"
            );

            for i in btokenids.clone() {
                ensure!(<Tokens<T>>::exists(i), "the btoken does not exist");

            }


            let nonce = <Nonce<T>>::get();
            let random_hash = (<system::Module<T>>::random_seed(), &sender, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            Self::_reserve(stokenid, sender.clone(), stotal);

            ensure!(!<SupplyOrderOwner<T>>::exists(random_hash), "Supply order already exists");

            let new_supply_order = SupplyOrder {
                id: random_hash,
                owner: sender.clone(),
                total: stotal,
                stoken: stokenid,
                tokens: btokenids,
                amortgage: amortgage,
                duration: duration,
                interest: interest,
            };

            let owned_supply_count = Self::owned_supply_count(&sender);
            let new_owned_supply_count = owned_supply_count.checked_add(1)
                .ok_or("Overflow add a new supply order to acount")?;

            let all_supply_order_count = Self::supply_order_count();
            let new_all_supply_order_count = all_supply_order_count.checked_add(1)
                .ok_or("Overflow adding a new supply order")?;

            <SupplyOrderDetail<T>>::insert(random_hash, new_supply_order);
            <SupplyOrderOwner<T>>::insert(random_hash, &sender);

            <AllSupplyOrder<T>>::insert(all_supply_order_count, random_hash);
            <AllSupplyOrderCount<T>>::put(new_all_supply_order_count);
            <AllSupplyOrderIndex<T>>::insert(random_hash, all_supply_order_count);

            <OwnedSupplyOrder<T>>::insert((sender.clone(), owned_supply_count), random_hash);
            <OwnedSupplyCount<T>>::insert(&sender, new_owned_supply_count);
            <OwnedSupplyIndex<T>>::insert(random_hash, owned_supply_count);


            <Nonce<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::CreateSupply(sender));

            Ok(())
        }




    }
}

impl<T: Trait> Module<T> {
     // the ERC20 standard transfer function
    // internal
    fn _transfer(
        token_id: T::AssetId,
        from: T::AccountId,
        to: T::AccountId,
        value: T::TokenBalance,
    ) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((token_id, from.clone())),
            "Account does not own this token"
        );

        let sender_balance = Self::balance_of((token_id, from.clone()));
        ensure!(sender_balance >= value, "Not enough balance.");

        let sender_free_balance = Self::free_balance_of((token_id, from.clone()));
        ensure!(sender_free_balance >= value, "Not enough free balance.");

        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;

        let updated_from_free_balance = sender_free_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating free balance")?;

        let receiver_balance = Self::balance_of((token_id, to.clone()));

        let receiver_free_balance = Self::free_balance_of((token_id, to.clone()));

        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;

        let updated_to_free_balance = receiver_free_balance
            .checked_add(&value)
            .ok_or("overflow in calculating free balance")?;

        // reduce sender's balance
        <BalanceOf<T>>::insert((token_id, from.clone()), updated_from_balance);
        <FreeBalanceOf<T>>::insert((token_id, from.clone()), updated_from_free_balance);

        // increase receiver's balance
        <BalanceOf<T>>::insert((token_id, to.clone()), updated_to_balance);
        <FreeBalanceOf<T>>::insert((token_id, to.clone()), updated_to_free_balance);

        Self::deposit_event(RawEvent::Transfer(token_id, from, to, value));
        Ok(())
    }

    fn _reserve(
        token_id: T::AssetId,
        sender: T::AccountId, 
        value: T::TokenBalance,
    ) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((token_id, sender.clone())),
            "Account does not own this token"
        );

        let sender_free_balance = Self::free_balance_of((token_id, sender.clone()));
        ensure!(sender_free_balance >= value, "Not enough free balance.");

        let sender_reserve_balance = Self::reserve_balance_of((token_id, sender.clone()));

        let updated_sender_free_balance = sender_free_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating reserve free balance")?;

        let updated_sender_reserve_balance = sender_reserve_balance
            .checked_add(&value)
            .ok_or("overflow in calculating reserve reserve balance")?;

        <FreeBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_free_balance);
        <ReserveBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_reserve_balance);

        Self::deposit_event(RawEvent::Reserve(token_id, sender, value));
        Ok(())

    }

    fn _unreserve(
        token_id: T::AssetId,
        sender: T::AccountId, 
        value: T::TokenBalance,
    ) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((token_id, sender.clone())),
            "Account does not own this token"
        );

        let sender_free_balance = Self::free_balance_of((token_id, sender.clone()));

        let sender_reserve_balance = Self::reserve_balance_of((token_id, sender.clone()));
        ensure!(sender_reserve_balance >= value, "Not enough reserve balance.");

        let updated_sender_free_balance = sender_free_balance
            .checked_add(&value)
            .ok_or("overflow in calculating unreserve free balance")?;

        let updated_sender_reserve_balance = sender_reserve_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating unreserve reserve balance")?;

        <FreeBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_free_balance);
        <ReserveBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_reserve_balance);

        Self::deposit_event(RawEvent::UnReserve(token_id, sender, value));
        Ok(())
    }
}
