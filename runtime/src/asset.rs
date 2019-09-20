use support::{decl_module, decl_storage, decl_event, StorageValue, dispatch::Result};
use system::ensure_signed;


#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Erc20Token<TokenBalance> {
    name: Vec<u8>,
    ticker: Vec<u8>,
    total_supply: TokenBalance,
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


decl_storage! {
	trait Store for Module<T: Trait> as SubstrateModuleTemplate {
		TokenId get(token_id) config(): T::AssetId;
        Tokens get(token_details): map T::AssetId => Erc20Token<T::TokenBalance>;
        BalanceOf get(balance_of): map (T::AssetId, T::AccountId) => T::TokenBalance;
        FreeBalanceOf get(free_balance_of): map (T::AssetId, T::AccountId) => T::TokenBalance;
        LockedBalanceOf get(locked_balance_of): map(T::AssetId, T::AccountId) => T::TokenBalance;

        Allowance get(allowance): map (T::AssetId, T::AccountId, T::AccountId) => T::TokenBalance;

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
            <LockedBalanceOf<T>>::insert((token_id, sender.clone()), T::TokenBalance::from(0u64));

            Ok(())
        }

        // 增发
        fn issue(origin, token_id: T::AssetId, added: T::TokenBalance) {
            let origin = ensure_signed(origin)?;

            ensure!(<Tokens<T>>::exists(token_id), "the token does not exist");
            ensure!(origin == Self::admin(), "only Admin can issue a token");

            let admin_balance = Self::balance_of((token_id, origin.clone()));
            let admin_free_balance = Self::free_balance_of((token_id, origin.clone()));
            let total = Self::token_details(token_id).total_supply;

            let admin_balance = admin_balance.checked_add(&added)
                .ok_or("overflow in calculating admin balance")?;
            let admin_free_balance = admin_free_balance.checked_add(&added)
                .ok_or("overflow in calculating admin free balance")?;
            let total = total.checked_add(&added)
                .ok_or("overflow in calculating total supply")?;

            let token = Erc20Token {
                name: Self::token_details(token_id).name,
                ticker: Self::token_details(token_id).ticker,
                total_supply: total,
            };

            <BalanceOf<T>>::insert((token_id, origin.clone()), admin_balance);
            <FreeBalanceOf<T>>::insert((token_id, origin.clone()), admin_free_balance);
            <Tokens<T>>::insert(token_id, token);

            Self::deposit_event(RawEvent::Issue(token_id, origin, total));
        }

        // 销毁
        fn burn(origin, token_id: T::AssetId, burned: T::TokenBalance) {
            let origin = ensure_signed(origin)?;
            ensure!(<Tokens<T>>::exists(token_id), "the token does not exist");
            ensure!(origin == Self::admin(), "only Admin can burn some token");

            let free_balance = Self::free_balance_of((token_id, origin.clone()));
            ensure!(free_balance >= burned, "origin free balance less than burned");

            let free_balance = free_balance.checked_sub(&burned)
                .ok_or("overflow in calculating free burn")?;

            let balance = Self::balance_of((token_id, origin.clone()));
            let balance = balance.checked_sub(&burned)
                .ok_or("overflow in calculting balance burn")?;

            let total = Self::token_details(token_id).total_supply;
            let total = total.checked_sub(&burned)
                .ok_or("overflow in calculating total supply")?;

            let token = Erc20Token {
                name: Self::token_details(token_id).name,
                ticker: Self::token_details(token_id).ticker,
                total_supply: total,
            };

            <BalanceOf<T>>::insert((token_id, origin.clone()), balance);
            <FreeBalanceOf<T>>::insert((token_id, origin.clone()), free_balance);
            <Tokens<T>>::insert(token_id, token);

            Self::deposit_event(RawEvent::Burn(token_id, origin, burned));
        }

        fn transfer(_origin, token_id: T::AssetId, to: T::AccountId, value: T::TokenBalance) -> Result {
            let sender = ensure_signed(_origin)?;
            Self::transfer(token_id, sender, to, value)
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


	}
}

decl_event!(
	pub enum Event<T>
    where
        <T as system::Trait>::AccountId,
        <T as self::Trait>::TokenBalance,
        <T as self::Trait>::AssetId,
    {
        Transfer(AssetId, AccountId, AccountId, TokenBalance),
        Approval(AssetId, AccountId, AccountId, TokenBalance),
		Issue(AssetId, AccountId, TokenBalance),
		Burn(AssetId, AccountId, TokenBalance),

        Lock(AssetId, AccountId, TokenBalance),
        UnLock(AssetId, AccountId, TokenBalance),
	}
);

impl<T: Trait> Module<T> {
    pub fn transfer(
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

    pub fn lock(token_id: T::AssetId, sender: T::AccountId, value: T::TokenBalance) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((token_id, sender.clone())),
            "Account does not own this token"
        );

        let sender_free_balance = Self::free_balance_of((token_id, sender.clone()));
        ensure!(sender_free_balance >= value, "Not enough free balance.");

        let sender_locked_balance = Self::locked_balance_of((token_id, sender.clone()));

        let updated_sender_free_balance = sender_free_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating locked free balance")?;

        let updated_sender_locked_balance = sender_locked_balance
            .checked_add(&value)
            .ok_or("overflow in calculating locked balance")?;

        <FreeBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_free_balance);
        <LockedBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_locked_balance);

        Self::deposit_event(RawEvent::Lock(token_id, sender, value));
        Ok(())
    }

    pub fn unlock(token_id: T::AssetId, sender: T::AccountId, value: T::TokenBalance) -> Result {
        ensure!(
            <BalanceOf<T>>::exists((token_id, sender.clone())),
            "Account does not own this token"
        );

        let sender_free_balance = Self::free_balance_of((token_id, sender.clone()));

        let sender_locked_balance = Self::locked_balance_of((token_id, sender.clone()));
        ensure!(
            sender_locked_balance >= value,
            "Not enough locked balance."
        );

        let updated_sender_free_balance = sender_free_balance
            .checked_add(&value)
            .ok_or("overflow in calculating unlock free balance")?;

        let updated_sender_locked_balance = sender_locked_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating unlock locked balance")?;

        <FreeBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_free_balance);
        <LockedBalanceOf<T>>::insert((token_id, sender.clone()), updated_sender_locked_balance);

        Self::deposit_event(RawEvent::UnLock(token_id, sender, value));
        Ok(())
    }
}