use crate::types::*;
use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PiersRugyardMinted {
    pub id: NonFungibleLocalId,
    pub nft_data: NFT,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PiersRugyardAuctionStarted {
    pub auction: Auction,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PiersRugyardAuctionSettled {
    pub auction: Auction,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PiersRugyardAuctionBid {
    pub auction: Auction,
    pub bid: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct PiersRugyardEarlyBuy {
    pub xrd_amount: Decimal,
    pub early_amount: Decimal,
}
