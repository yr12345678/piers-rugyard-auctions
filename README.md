The blueprint for the Piers Rugyard NFT collection minting and auction component. Use it for inspiration, or just to check if I can't rug you.

## Components

* Stokenet: `component_tdx_2_1czvpp7w2htcj39n8qxkxew62ltaz6aaz97206vmahgdyg54fwnyekz`
* Mainnet: `component_rdx1cq86467tkaj0dsjavw2asjwugw5rggp3nf7hs3z30577xlj6nv22wr`

## How it works

* Piers Rugyard NFTs are minted by the owner
* Owner sets the component to active to allow auctions to start and starts the first auction
* The auction has a duration and a buffer (time before auction ends). If a bid is made in the buffer, the auction is extended. 
* Accounts can bid on the NFT with XRD. Note that it's not possible to retract a bid.
* If the new bid is valid, the previous bid is immediately returned to the original account.
* Once the auction ends, bidding is no longer possible and the auction must be settled. Settling an auction is incentivized with 5% of the winning bid for the settler to keep things moving smoothly.
    * If an auction ends without bids, the first bidder after the auction ended will be the winner of the auction. The auction will be settled immediately.
* After an auction is settled and there is another NFT available to be auctioned, a new auction will start automatically.

An account locker is used to store/route reward and NFT deposits.

## Types

### Auction
* `id`: u64,
* `start_timestamp`: Instant
* `end_timestamp`: Instant
* `nft`: NonFungibleLocalId
* `highest_bid`: Option\<Decimal\>
* `highest_bidder`: Option<Global\<Account\>>
* `bid_count`: u64
* `latest_bids`: Vec\<Bid\> (contains 10 latest bids)

### NFT
* `key_image_url`: Url
* `name`: String

### Bid
* `amount`: Decimal
* `bidder`: Global<Account>
* `timestamp`: Instant
* `transaction_hash`: Hash

## Events

### PiersRugyardMinted
* `id`: NonFungibleLocalId
* `nft_data`: NFT

### PiersRugyardAuctionStarted
* `auction`: Auction

### PiersRugyardAuctionSettled
* `auction`: Auction

### PiersRugyardAuctionBid
* `auction`: Auction
* `bid`: Bid
