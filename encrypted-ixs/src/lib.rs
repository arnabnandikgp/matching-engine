use arcis_imports::*;


/// the vault of the escrow account will have the following seeds:

#[encrypted]
mod circuits {
    use arcis_imports::*;

    pub const MAX_ORDERS: usize = 100;
    pub const MAX_MATCHES_PER_BATCH: usize = 10;

    #[derive(Copy, Clone)]
    pub struct Order {
        pub order_id: u64,
        pub user_id: [u8; 32],
        pub base_mint: [u8; 32],
        pub quote_mint: [u8; 32],
        pub amount: u64,
        pub price: u64,
        pub order_type: u8,
        pub timestamp: u64,
    }

    impl Order {
        pub fn empty() -> Self {
            Order {
                order_id: 0,
                user_id: [0u8; 32],
                base_mint: [0u8; 32],
                quote_mint: [0u8; 32],
                amount: 0,
                price: 0,
                order_type: 0,
                timestamp: 0,
            }
        }

        pub fn is_buy(&self) -> bool {
            self.order_type == 0
        }

        pub fn is_sell(&self) -> bool {
            self.order_type == 1
        }
    }

    pub struct OrderBook {
        pub buy_orders: [Order; MAX_ORDERS],
        pub buy_count: usize,
        pub sell_orders: [Order; MAX_ORDERS],
        pub sell_count: usize,
    }


    #[instruction]
    pub fn match_orders(_input_ctxt: Enc<Shared, u8>) -> Enc<Shared, MatchResult> {
        let mut order_book = get_order_book();
        let mut result = MatchResult {
            matches: [MatchedOrder::empty(); MAX_MATCHES_PER_BATCH],
            num_matches: 0,
            match_ids: [[0u8; 32]; MAX_MATCHES_PER_BATCH],
            buyer_ids: [[0u8; 32]; MAX_MATCHES_PER_BATCH],
            seller_ids: [[0u8; 32]; MAX_MATCHES_PER_BATCH],
            base_mints: [[0u8; 32]; MAX_MATCHES_PER_BATCH],
            quote_mints: [[0u8; 32]; MAX_MATCHES_PER_BATCH],
            quantities: [0u64; MAX_MATCHES_PER_BATCH],
            execution_prices: [0u64; MAX_MATCHES_PER_BATCH],
        };

        let mut match_count = 0usize;
        let mut next_match_id = 0u64;

        for _ in 0..MAX_MATCHES_PER_BATCH {
            // Check if we can match - replace match expression with if-else
            let can_match = {
                let has_both = order_book.has_buy() && order_book.has_sell();
                
                if has_both {
                    let buy = order_book.peek_buy();
                    let sell = order_book.peek_sell();
                    buy.price >= sell.price
                } else {
                    false
                }
            };

            if !can_match {
                // Stop matching
            } else {
                let buyer_opt = order_book.pop_buy();
                let seller_opt = order_book.pop_sell();
                
                if buyer_opt.is_some() && seller_opt.is_some() {
                    let mut buyer = buyer_opt.unwrap();
                    let mut seller = seller_opt.unwrap();

                    let execution_price = (buyer.price + seller.price) / 2;
                    let fill_quantity = if buyer.amount < seller.amount {
                        buyer.amount
                    } else {
                        seller.amount
                    };

                    let match_id = next_match_id;
                    next_match_id = next_match_id + 1;

                    result.matches[match_count] = MatchedOrder {
                        match_id,
                        buyer_id: buyer.user_id,
                        seller_id: seller.user_id,
                        base_mint: buyer.base_mint,
                        quote_mint: buyer.quote_mint,
                        quantity: fill_quantity,
                        execution_price,
                    };

                    result.match_ids[match_count] = u64_to_bytes(match_id);
                    result.buyer_ids[match_count] = buyer.user_id;
                    result.seller_ids[match_count] = seller.user_id;
                    result.base_mints[match_count] = buyer.base_mint;
                    result.quote_mints[match_count] = buyer.quote_mint;
                    result.quantities[match_count] = fill_quantity;
                    result.execution_prices[match_count] = execution_price;

                    buyer.amount = buyer.amount - fill_quantity;
                    seller.amount = seller.amount - fill_quantity;

                    if buyer.amount > 0 {
                        order_book.insert_buy(buyer);
                    }

                    if seller.amount > 0 {
                        order_book.insert_sell(seller);
                    }

                    match_count = match_count + 1;
                }
            }
        }

        result.num_matches = match_count as u64;
        _input_ctxt.owner.from_arcis(result)
    }

    fn u64_to_bytes(val: u64) -> [u8; 32] {
        let mut result = [0u8; 32];
        let bytes = val.to_le_bytes();
        result[0] = bytes[0];
        result[1] = bytes[1];
        result[2] = bytes[2];
        result[3] = bytes[3];
        result[4] = bytes[4];
        result[5] = bytes[5];
        result[6] = bytes[6];
        result[7] = bytes[7];
        result
    }

    
}
