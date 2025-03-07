The blueprint for the Piers Rugyard NFT collection minting and auction component. Use it for inspiration, or just to check if I can't rug you.

## Components

* Stokenet: `component_tdx_2_1cz3r88mksf55qf7avskt2checjgaec2edrvfjph9gvdp3gusvqm88p`
* Mainnet: not yet deployed

## How it works

* Piers Rugyard NFTs are minted by the owner
* Owner sets the component to active to allow auctions to start and starts the first auction
* The auction has a duration and a buffer (time before auction ends). If a bid is made in the buffer, the auction is extended. 
* Accounts can bid on the NFT with XRD
* Once the auction ends, bidding is no longer possible and the auction must be settled. Settling an auction is incentivized with 5% of the winning bid for the settler to keep things moving smoothly.
    * If an auction ends without bids, the first bidder after the auction ended will be the winner of the auction. The auction will be settled immediately.
* After an auction is settled and there is another NFT available to be auctioned, a new auction will start automatically.

An account locker is used to store/route reward and NFT deposits.

