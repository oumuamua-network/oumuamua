        OwnedOrder get(owned_order): map (T::AccountId, T::OrderType, u64) => T::Hash;
        OwnedCount get(owned_count): map (T::AccountId, T::OrderType) => u64;
        OrderTokenCount get(order_token_count): map (T::AccountId, T::OrderType) => u64;
        OrderTokenKind get(order_token_kind): map (T::AccountId, T::OrderType, u64) => T::AssetId;
        OrderTokenTotal get(order_token_total): map (T::AccountId, T::OrderType, T::AssetId) => T::TokenBalance;