use events::*;
use scrypto::prelude::*;
use types::*;

pub mod events;
pub mod types;

#[blueprint]
#[types(u64, Auction, NFT)]
#[events(
    PiersRugyardAuctionBid,
    PiersRugyardAuctionSettled,
    PiersRugyardAuctionStarted,
    PiersRugyardEarlyBuy,
    PiersRugyardMinted
)]
mod piers_rugyard {
    enable_method_auth! {
        methods {
            settle_auction => PUBLIC;
            start_new_auction => PUBLIC;
            bid => PUBLIC;
            mint_nfts => restrict_to: [OWNER];
            withdraw_profits => restrict_to: [OWNER];
            update_pool_address => restrict_to: [OWNER];
            update_auction_duration => restrict_to: [OWNER];
            update_auction_buffer => restrict_to: [OWNER];
            update_minimum_bid_increase => restrict_to: [OWNER];
            delete_nft => restrict_to: [OWNER];
            flip_status => restrict_to: [OWNER];
            get_current_auction => PUBLIC;
            get_completed_auction => PUBLIC;
            get_profit_amount => PUBLIC;
            deposit_xrd_domain => restrict_to: [OWNER];
            withdraw_xrd_domain => restrict_to: [OWNER];
        }
    }

    struct PiersRugyard {
        early_pool: ComponentAddress,
        early_address: ResourceAddress,
        auction_duration_minutes: u64,
        auction_buffer_minutes: u64,
        completed_auctions: KeyValueStore<u64, Auction>,
        current_auction: Option<Auction>,
        available_nfts_vault: NonFungibleVault,
        highest_bid_vault: FungibleVault,
        early_vault: FungibleVault,
        total_early_bought: Decimal,
        minimum_bid_increase: Decimal,
        locker: Global<AccountLocker>,
        owner_resource: ResourceAddress,
        active: bool,
        nft_manager: NonFungibleResourceManager,
        available_nfts_list: Vec<NonFungibleLocalId>,
        next_nft_id: u64,
        next_auction_id: u64,
        xrd_domain_resource: ResourceAddress,
        xrd_domain_vault: NonFungibleVault,
    }

    impl PiersRugyard {
        pub fn instantiate(
            auction_duration_minutes: u64,
            auction_buffer_minutes: u64,
            minimum_bid_increase: Decimal,
            owner_resource: ResourceAddress,
            early_pool: ComponentAddress,
            early_address: ResourceAddress,
            xrd_domain_resource: ResourceAddress,
        ) -> Global<PiersRugyard> {
            // Get the component address
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(PiersRugyard::blueprint_id());

            // Set some rules
            let owner_rule = rule!(require(owner_resource));
            let global_caller_rule = rule!(require(global_caller(component_address)));

            // Create the NFT collection
            let nft_manager =
                ResourceBuilder::new_integer_non_fungible_with_registered_type::<NFT>(
                    OwnerRole::Fixed(owner_rule.clone()),
                )
                .mint_roles(mint_roles!(
                    minter => global_caller_rule.clone();
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles!(
                    burner => global_caller_rule.clone();
                    burner_updater => rule!(deny_all);
                ))
                .metadata(metadata!(
                    roles {
                        metadata_setter => OWNER;
                        metadata_setter_updater => OWNER;
                        metadata_locker => OWNER;
                        metadata_locker_updater => OWNER;
                    },
                    init {
                        "name" => "Piers Rugyard", locked;
                        "description" => "The official Piers Rugyard NFT collection. Piers' legacy continues in your wallet.", updatable;
                        "icon_url" => "https://www.google.com", updatable;
                    }
                ))
                .create_with_no_initial_supply();

            // Instantiate the account locker used to deposit losing bids and won NFTs
            let account_locker = Blueprint::<AccountLocker>::instantiate(
                OwnerRole::Fixed(owner_rule.clone()),
                global_caller_rule.clone(),
                global_caller_rule.clone(),
                global_caller_rule.clone(),
                global_caller_rule.clone(),
                None,
            );

            // Instantiate and globalize
            Self {
                early_pool,
                auction_duration_minutes,
                auction_buffer_minutes,
                completed_auctions: KeyValueStore::<u64, Auction>::new_with_registered_type(),
                current_auction: None,
                available_nfts_vault: NonFungibleVault::new(nft_manager.address()),
                highest_bid_vault: FungibleVault::new(XRD),
                early_vault: FungibleVault::new(early_address),
                total_early_bought: dec!(0),
                minimum_bid_increase,
                locker: account_locker,
                owner_resource,
                active: false,
                early_address,
                nft_manager,
                available_nfts_list: Vec::new(),
                next_nft_id: 1,
                next_auction_id: 1,
                xrd_domain_resource,
                xrd_domain_vault: NonFungibleVault::new(xrd_domain_resource),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Updatable(owner_rule.clone()))
            .with_address(address_reservation)
            .globalize()
        }

        /// Starts a new auction of one is not already active. We also make sure
        /// there is actually an NFT to auction and the auction system is active.
        ///
        /// # Panics
        /// * An auction is already active
        /// * There are no NFTs to auction
        pub fn start_new_auction(&mut self) {
            // Make sure there's not already an auction active
            assert!(
                self.current_auction.is_none(),
                "There's already an auction active!"
            );

            // Make sure we have at least 1 NFT to auction
            assert!(
                !self.available_nfts_list.is_empty(),
                "No NFTs left to auction!"
            );

            // Make sure we are allowed to start new auctions at the moment
            assert!(self.active, "Auctions are not active at the moment!");

            // Calculate the start and end timestamps
            let start_timestamp = Clock::current_time(TimePrecisionV2::Second);
            let end_timestamp = start_timestamp
                .add_minutes(self.auction_duration_minutes as i64)
                .expect("Could not calculate end timestamp");

            // Get the next NFT local id for the auction
            let nft_local_id = self.available_nfts_list.remove(0);

            // Create the auction struct and assign it as the current auction
            let auction = Auction {
                id: self.next_auction_id,
                start_timestamp,
                end_timestamp,
                nft: nft_local_id.clone(),
                highest_bid: None,
                highest_bidder: None,
                bid_count: 0,
                bid_history: IndexMap::new()
            };

            self.current_auction = Some(auction.clone());

            // Emit event
            Runtime::emit_event(PiersRugyardAuctionStarted { auction });

            // Increment the auction id for the next auction
            self.next_auction_id += 1;
        }

        /// Places a new bid on the currently auctioned NFT. A new bid must be higher than the
        /// current highest bid with a minimum increase. The bid must also be placed before the
        /// auction has ended, except when no previous bid was present, in which case the first
        /// bid will automatically be the winning bid. When the new bid is valid, the previous
        /// highest bid gets returned to the originating account.
        ///
        /// If a bid happens shortly before the auction ends, the auction gets extended. This
        /// keeps happening until the auction ends without new bids shortly before it.
        ///
        /// # Input
        /// * `bid`: a Bucket containing the resource the bid is done with
        /// * `account`: a Global<Account> so we can register which account made the bid
        ///
        /// # Panics
        /// * No auction is active
        /// * Bid resource is invalid
        /// * Bid increase is too low
        /// * Bid happens after auction ended while there was already a bid
        pub fn bid(&mut self, bid: Bucket, account: Global<Account>) -> (Option<FungibleBucket>, Option<NonFungibleBucket>) {
            // Ensure the caller owns the account
            Runtime::assert_access_rule(account.get_owner_role().rule);

            let current_timestamp = Clock::current_time(TimePrecisionV2::Second);
            let auction: &mut Auction = self.current_auction.as_mut().expect("No auction active!");
            let highest_bid_amount = auction.highest_bid.unwrap_or(dec!(0));

            // Add the bid to the bid history
            let new_bid = Bid {
                amount: bid.amount(),
                bidder: account,
                timestamp: current_timestamp,
                transaction_hash: Runtime::transaction_hash(),
            };

            auction.bid_count += 1;
            auction.bid_history.insert(auction.bid_count, new_bid.clone());

            // Emit event here so we can clone `auction`
            Runtime::emit_event(PiersRugyardAuctionBid {
                auction: auction.clone(),
                bid: new_bid,
            });

            // Assert the bid is valid
            assert!(bid.resource_address() == XRD, "You must bid with XRD!");
            assert!(
                bid.amount() - highest_bid_amount >= self.minimum_bid_increase,
                "Bid increase not high enough!"
            );

            // If we already have a bid, ensure we can still accept new bids
            // and return the previous bid.
            let mut first_bidder = false;
            if let Some(highest_bidder) = auction.highest_bidder {
                assert!(
                    current_timestamp < auction.end_timestamp,
                    "Auction has ended!"
                );

                let old_bid = self.highest_bid_vault.take_all();
                self.locker.store(highest_bidder, old_bid.into(), true);
            } else {
                first_bidder = true;
            }

            // Extend the auction if necessary
            let current_timestamp_plus_buffer = current_timestamp
                .add_minutes(self.auction_buffer_minutes as i64)
                .expect("Could not add minutes!");

            if current_timestamp_plus_buffer >= auction.end_timestamp
                && current_timestamp < auction.end_timestamp
            {
                auction.end_timestamp = current_timestamp_plus_buffer;
            }

            // Accept the new bid
            auction.highest_bid = Some(bid.amount());
            auction.highest_bidder = Some(account);
            self.highest_bid_vault.put(bid.as_fungible());

            // If this was the first bid AND the auction has ended, we might as well settle it immediately
            if first_bidder && current_timestamp >= auction.end_timestamp {
                info!("Settling auction");
                let (reward, nft) = self.settle_auction(account);

                (Some(reward), nft)
            } else {
                (None, None)
            }
        }

        /// Settles an auction if there is one that has ended. Whoever calls
        /// this method will get a 5% reward for settling the auction.
        ///
        /// Profits from the auction are then swapped to EARLY. If it's possible
        /// to start a new auction, this will be done immediately.
        ///
        /// # Input
        /// * `account`: A Global<Account> so we know where to send the reward to
        ///
        /// # Panics
        /// * Auction has not ended yet
        /// * There is no bid yet
        pub fn settle_auction(&mut self, account: Global<Account>) -> (FungibleBucket, Option<NonFungibleBucket>) {
            // Ensure the caller owns the account
            Runtime::assert_access_rule(account.get_owner_role().rule);

            let auction = self.current_auction.as_mut().expect("No auction active!");
            let current_timestamp = Clock::current_time(TimePrecisionV2::Second);

            // Emit event
            Runtime::emit_event(PiersRugyardAuctionSettled {
                auction: auction.clone(),
            });

            // Make sure auction time has passed
            assert!(
                current_timestamp >= auction.end_timestamp,
                "Current auction has not ended yet"
            );

            // Make sure we have a bidder
            assert!(
                auction.highest_bid.is_some(),
                "No bids were made. Wait until at least 1 bid was made."
            );

            // Deposit NFT to the winner. If the current caller is the winner, give it to them directly
            let nft = self.available_nfts_vault.take_non_fungible(&auction.nft);
            let mut nft_bucket: Option<NonFungibleBucket> = None;
            if auction.highest_bidder.unwrap() == account {
                nft_bucket = Some(nft);
            } else {
                self.locker
                .store(auction.highest_bidder.unwrap(), nft.into(), true);
            }

            // Take the reward for the account calling this method
            let mut highest_bid_bucket = self.highest_bid_vault.take_all();
            let reward = highest_bid_bucket
                .amount()
                .checked_mul(dec!(0.05))
                .expect("Couldn't calculate reward!");
            let reward_bucket = highest_bid_bucket
                .take_advanced(reward, WithdrawStrategy::Rounded(RoundingMode::ToZero));

            // Swap for EARLY and deposit
            let highest_bid_amount = highest_bid_bucket.amount();

            let pool_component: Global<AnyComponent> = Global::from(self.early_pool);
            let early_bucket =
                pool_component.call_raw::<Bucket>("swap", scrypto_args!(highest_bid_bucket));

            Runtime::emit_event(PiersRugyardEarlyBuy {
                xrd_amount: highest_bid_amount,
                early_amount: early_bucket.amount(),
            });

            self.total_early_bought += early_bucket.amount();
            self.early_vault.put(early_bucket.as_fungible());

            // Settle the auction
            self.completed_auctions.insert(auction.id, auction.clone());
            self.current_auction = None;

            // Start new auction if possible
            if !self.available_nfts_list.is_empty() && self.active {
                self.start_new_auction();
            }

            (reward_bucket, nft_bucket)
        }

        //------ Admin stuff ------//

        /// Withdraws the profits from the EARLY vault
        pub fn withdraw_profits(&mut self) -> FungibleBucket {
            self.early_vault.take_all()
        }

        /// Updates the pool address used for swapping
        ///
        /// # Input
        /// * `address`: a ComponentAddress of the new pool
        pub fn update_pool_address(&mut self, address: ComponentAddress) {
            self.early_pool = address;
        }

        /// Updates the auction duration
        ///
        /// # Input
        /// * `minutes`: a u64 for the new auction duration in minutes
        ///
        /// # Panics
        /// * Duration is 0 or lower
        /// * Duration is shorter or equal to time buffer
        pub fn update_auction_duration(&mut self, minutes: u64) {
            assert!(minutes > 0, "Auction duration must be more than 0 minutes!");
            assert!(
                minutes > self.auction_buffer_minutes,
                "Auction duration must be longer than the auction buffer!"
            );

            self.auction_duration_minutes = minutes;
        }

        /// Updates the auction buffer
        ///
        /// # Input
        /// * `minutes`: a u64 for the new time buffer in minutes
        ///
        /// # Panics
        /// * The buffer is 0 or lower
        /// * The buffer is higher than the auction duration
        pub fn update_auction_buffer(&mut self, minutes: u64) {
            assert!(minutes > 0, "Buffer must be more than 0 minutes!");
            assert!(
                minutes < self.auction_duration_minutes,
                "Buffer must be lower than the auction duration!"
            );

            self.auction_buffer_minutes = minutes;
        }

        /// Upates the minimum bid increase
        ///
        /// # Input
        /// * `minimum_bid_increase`: A Decimal for the new minimum bid increase
        ///
        /// # Panics
        /// * The minimum bid increase is 0 or lower
        pub fn update_minimum_bid_increase(&mut self, minimum_bid_increase: Decimal) {
            assert!(
                minimum_bid_increase > dec!(0),
                "Minimum bid increase must be higher than 0!"
            );

            self.minimum_bid_increase = minimum_bid_increase;
        }

        /// Activates or deactives the auction system
        pub fn flip_status(&mut self) {
            self.active = !self.active;
        }

        /// Mints a new NFT for the collection and puts it in the list
        /// of NFTs to be auctioned.
        ///
        /// # Input
        /// * `nft_data`: an NFT struct with the data for the new NFT        
        pub fn mint_nfts(&mut self, nft_data: Vec<NFT>) {
            for data in nft_data {
                let local_id = NonFungibleLocalId::integer(self.next_nft_id);
                let nft = self.nft_manager.mint_non_fungible(&local_id, data.clone());

                // Put NFT in vault and in available NFTs list
                self.available_nfts_list.push(local_id.clone());
                self.available_nfts_vault.put(nft);

                Runtime::emit_event(PiersRugyardMinted {
                    id: local_id,
                    nft_data: data,
                });

                // Increment the NFT id
                self.next_nft_id += 1;
            }
        }

        /// Removes an NFT from the available NFTs list and burns it. This method is protected.
        ///
        /// # Input
        /// * `id`: A NonFungibleLocalId for the NFT to be deleted
        ///
        /// # Panics
        /// * NFT does not exist
        /// * NFT is currently under auction
        pub fn delete_nft(&mut self, id: NonFungibleLocalId) {
            assert!(
                self.available_nfts_list.contains(&id),
                "NFT is not available!"
            );

            if let Some(auction) = &self.current_auction {
                assert!(
                    auction.nft != id,
                    "Can't delete an NFT that's currently under auction!"
                );
            }

            // Burn the NFT and remove it from the list
            let nft_position = self
                .available_nfts_list
                .iter()
                .position(|nft| nft == &id)
                .expect("Could not find NFT!");
            self.available_nfts_list.remove(nft_position);
            self.available_nfts_vault.take_non_fungible(&id).burn();
        }

        /// Deposits an XRD domain into the vault
        /// 
        /// # Input
        /// * `domain`: a NonFungibleBucket containing the XRD Domain
        /// 
        /// # Panics
        /// * The resource address is incorrect
        pub fn deposit_xrd_domain(&mut self, domain: NonFungibleBucket) {
            assert!(domain.resource_address() == self.xrd_domain_resource, "Not an XRD Domain");

            self.xrd_domain_vault.put(domain);
        }

        /// Withdraws an XRD domain from the vault
        /// 
        /// # Input
        /// * `local_id`: the NonFungibleLocalId of the XRD Domain
        /// 
        /// # Output
        /// * A NonFungibleBucket containing the XRD Domain
        /// 
        /// # Panics
        /// * The domain is not present in the vault
        pub fn withdraw_xrd_domain(&mut self, local_id: NonFungibleLocalId) -> NonFungibleBucket {
            self.xrd_domain_vault.take_non_fungibles(&indexset!(local_id))
        }        

        //------ Getters ------//
        /// TODO - Add more getters
        /// Returns the current auction or None
        pub fn get_current_auction(&mut self) -> Option<Auction> {
            self.current_auction.clone()
        }

        /// Gets a completed auction by its id
        pub fn get_completed_auction(&mut self, id: u64) -> Auction {
            self.completed_auctions.get(&id).unwrap().clone()
        }

        /// Get the amount of profit
        pub fn get_profit_amount(&mut self) -> Decimal {
            self.early_vault.amount()
        }
    }
}
