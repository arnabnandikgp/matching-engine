use arcis_imports::*;

#[encrypted(network = "localnet")]
mod circuits {
    use arcis_imports::*;

    pub const MAX_ORDERS: usize = 10;
    pub const MAX_MATCHES_PER_BATCH: usize = 5;

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

    impl OrderBook {
        pub fn new() -> Self {
            OrderBook {
                buy_orders: [Order::empty(); MAX_ORDERS],
                buy_count: 0,
                sell_orders: [Order::empty(); MAX_ORDERS],
                sell_count: 0,
            }
        }

        pub fn insert_buy(&mut self, order: Order) -> bool {
            let success = if self.buy_count >= MAX_ORDERS {
                false
            } else {
                self.buy_orders[self.buy_count] = order;
                self.buy_count += 1;
                
                let mut i = self.buy_count - 1;
                let mut done = false;
                for _ in 0..MAX_ORDERS {
                    if i == 0 || done {
                        done = true;
                    } else {
                        let parent = (i - 1) / 2;
                        if self.compare_buy(i, parent) {
                            self.buy_orders.swap(i, parent);
                            i = parent;
                        } else {
                            done = true;
                        }
                    }
                }
                
                true
            };
            success
        }

        pub fn insert_sell(&mut self, order: Order) -> bool {
            let success = if self.sell_count >= MAX_ORDERS {
                false
            } else {
                self.sell_orders[self.sell_count] = order;
                self.sell_count += 1;
                
                let mut i = self.sell_count - 1;
                let mut done = false;
                for _ in 0..MAX_ORDERS {
                    if i == 0 || done {
                        done = true;
                    } else {
                        let parent = (i - 1) / 2;
                        if self.compare_sell(i, parent) {
                            self.sell_orders.swap(i, parent);
                            i = parent;
                        } else {
                            done = true;
                        }
                    }
                }
                
                true
            };
            success
        }

        fn compare_buy(&self, i: usize, j: usize) -> bool {
            let a = &self.buy_orders[i];
            let b = &self.buy_orders[j];
            
            if a.price != b.price {
                a.price > b.price
            } else {
                a.timestamp < b.timestamp
            }
        }

        fn compare_sell(&self, i: usize, j: usize) -> bool {
            let a = &self.sell_orders[i];
            let b = &self.sell_orders[j];
            
            if a.price != b.price {
                a.price < b.price
            } else {
                a.timestamp < b.timestamp
            }
        }

        fn heapify_buy(&mut self, mut i: usize) {
            let mut done = false;
            for _ in 0..MAX_ORDERS {
                if done {
                    // continue looping until done
                } else {
                    let left = 2 * i + 1;
                    let right = 2 * i + 2;
                    let mut largest = i;

                    if left < self.buy_count && self.compare_buy(left, largest) {
                        largest = left;
                    }

                    if right < self.buy_count && self.compare_buy(right, largest) {
                        largest = right;
                    }

                    if largest != i {
                        self.buy_orders.swap(i, largest);
                        i = largest;
                    } else {
                        done = true;
                    }
                }
            }
        }

        fn heapify_sell(&mut self, mut i: usize) {
            let mut done = false;
            for _ in 0..MAX_ORDERS {
                if done {
                    // continue looping until done
                } else {
                    let left = 2 * i + 1;
                    let right = 2 * i + 2;
                    let mut smallest = i;

                    if left < self.sell_count && self.compare_sell(left, smallest) {
                        smallest = left;
                    }

                    if right < self.sell_count && self.compare_sell(right, smallest) {
                        smallest = right;
                    }

                    if smallest != i {
                        self.sell_orders.swap(i, smallest);
                        i = smallest;
                    } else {
                        done = true;
                    }
                }
            }
        }

        pub fn pop_buy(&mut self) -> Option<Order> {
            let result = if self.buy_count == 0 {
                None
            } else {
                let order = self.buy_orders[0];
                self.buy_count -= 1;

                if self.buy_count > 0 {
                    self.buy_orders[0] = self.buy_orders[self.buy_count];
                    self.heapify_buy(0);
                }

                Some(order)
            };
            result
        }

        pub fn pop_sell(&mut self) -> Option<Order> {
            let result = if self.sell_count == 0 {
                None
            } else {
                let order = self.sell_orders[0];
                self.sell_count -= 1;

                if self.sell_count > 0 {
                    self.sell_orders[0] = self.sell_orders[self.sell_count];
                    self.heapify_sell(0);
                }

                Some(order)
            };
            result
        }

        pub fn peek_buy(&self) -> Order {
            if self.buy_count > 0 {
                self.buy_orders[0]
            } else {
                Order::empty()
            }
        }

        pub fn peek_sell(&self) -> Order {
            if self.sell_count > 0 {
                self.sell_orders[0]
            } else {
                Order::empty()
            }
        }
        
        pub fn has_buy(&self) -> bool {
            self.buy_count > 0
        }
        
        pub fn has_sell(&self) -> bool {
            self.sell_count > 0
        }
    }

    #[derive(Copy, Clone)]
    pub struct MatchedOrder {
        pub match_id: u64,
        pub buyer_id: [u8; 32],
        pub seller_id: [u8; 32],
        pub base_mint: [u8; 32],
        pub quote_mint: [u8; 32],
        pub quantity: u64,
        pub execution_price: u64,
    }

    impl MatchedOrder {
        pub fn empty() -> Self {
            MatchedOrder {
                match_id: 0,
                buyer_id: [0u8; 32],
                seller_id: [0u8; 32],
                base_mint: [0u8; 32],
                quote_mint: [0u8; 32],
                quantity: 0,
                execution_price: 0,
            }
        }
    }

    pub struct MatchResult {
        pub matches: [MatchedOrder; MAX_MATCHES_PER_BATCH],
        pub num_matches: u64,
        pub match_ids: [[u8; 32]; MAX_MATCHES_PER_BATCH],
        pub buyer_ids: [[u8; 32]; MAX_MATCHES_PER_BATCH],
        pub seller_ids: [[u8; 32]; MAX_MATCHES_PER_BATCH],
        pub base_mints: [[u8; 32]; MAX_MATCHES_PER_BATCH],
        pub quote_mints: [[u8; 32]; MAX_MATCHES_PER_BATCH],
        pub quantities: [u64; MAX_MATCHES_PER_BATCH],
        pub execution_prices: [u64; MAX_MATCHES_PER_BATCH],
    }

    // Note: Without static mut support, we create a new OrderBook for each call
    // In production, we would need to persist order book state on-chain
    fn get_order_book() -> OrderBook {
        OrderBook::new()
    }

    #[instruction]
    pub fn submit_order(
        order_id: u64,
        timestamp: u64,
        encrypted_order_ctxt: Enc<Shared, Order>,
    ) -> Enc<Shared, bool> {
        let mut order = encrypted_order_ctxt.to_arcis();
        order.order_id = order_id;
        order.timestamp = timestamp;

        let mut order_book = get_order_book();
        let success = if order.is_buy() {
            order_book.insert_buy(order)
        } else {
            order_book.insert_sell(order)
        };

        encrypted_order_ctxt.owner.from_arcis(success)
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
