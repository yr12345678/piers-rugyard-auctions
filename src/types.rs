use scrypto::prelude::*;

#[derive(ScryptoSbor, PartialEq, Debug, Clone)]
pub struct Auction {
    pub id: u64,
    pub start_timestamp: Instant,
    pub end_timestamp: Instant,
    pub nft: NonFungibleLocalId,
    pub highest_bid: Option<Decimal>,
    pub highest_bidder: Option<Global<Account>>,
}

#[derive(ScryptoSbor, NonFungibleData, Debug, PartialEq, Clone)]
pub struct NFT {
    pub key_image_url: Url,
    pub name: String,
}
