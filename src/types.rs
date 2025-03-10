use scrypto::prelude::*;

#[derive(ScryptoSbor, PartialEq, Debug, Clone)]
pub struct Auction {
    pub id: u64,
    pub start_timestamp: Instant,
    pub end_timestamp: Instant,
    pub nft: NonFungibleLocalId,
    pub highest_bid: Option<Decimal>,
    pub highest_bidder: Option<Global<Account>>,
    pub bid_count: u64,
    pub latest_bids: Vec<Bid>
}

#[derive(ScryptoSbor, PartialEq, Debug, Clone)]
pub struct Bid {
    pub amount: Decimal,
    pub bidder: Global<Account>,
    pub timestamp: Instant,
    pub transaction_hash: Hash
}

#[derive(ScryptoSbor, NonFungibleData, Debug, PartialEq, Clone)]
pub struct NFT {
    pub key_image_url: Url,
    pub name: String,
}
