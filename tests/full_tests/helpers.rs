use piers_rugyard::types::*;
use scrypto_test::prelude::*;

pub struct Account {
    pub public_key: Secp256k1PublicKey,
    pub private_key: Secp256k1PrivateKey,
    pub address: ComponentAddress,
}

#[derive(ScryptoSbor, NonFungibleData, Debug, PartialEq, Clone)]
pub struct XRDDomain {}

// Creates a new account
pub fn create_account(ledger: &mut DefaultLedgerSimulator) -> Account {
    let (public_key, private_key, address) = ledger.new_allocated_account();

    Account {
        public_key,
        private_key,
        address,
    }
}

// Just gets a shitton of XRD
pub fn get_shitton_of_xrd(ledger: &mut DefaultLedgerSimulator, account: &Account) {
    for _ in 0..100 {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .deposit_entire_worktop(account.address)
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        receipt.expect_commit_success();
    }
}

// Creates and environment with a deployed pool and PiersRugyard component
pub fn create_test_environment() -> (
    DefaultLedgerSimulator,
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
    Account,
) {
    // Setup the environment
    let mut ledger = LedgerSimulatorBuilder::new()
        .with_custom_protocol(|builder| builder.from_current_to_latest())
        .without_kernel_trace()
        .build();

    let account = create_account(&mut ledger);
    get_shitton_of_xrd(&mut ledger, &account);

    // Create mock EARLY and deposit into account
    let early_resource = ledger.create_fungible_resource(dec!(100_000), 18, account.address);

    // Publish mock Oci pool package
    let oci_package_address = ledger.compile_and_publish("mock_oci_pool");

    // Instantiate Oci pool package
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account.address, early_resource, dec!(100_000))
        .take_all_from_worktop(early_resource, "early_bucket")
        .withdraw_from_account(account.address, XRD, dec!(100_000))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .call_function_with_name_lookup(
            oci_package_address,
            "MockOciPool",
            "instantiate",
            |lookup| (lookup.bucket("xrd_bucket"), lookup.bucket("early_bucket")),
        )
        .deposit_batch(account.address, ManifestExpression::EntireWorktop)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    let pool_component = receipt.expect_commit_success().new_component_addresses()[0];

    // Create mock owner resource
    let owner_resource = ledger.create_fungible_resource(dec!(1), 0, account.address);

    // Create mock XRD Domain
    let xrd_domain = ledger.create_non_fungible_resource(account.address);

    // Publish NFT package
    let nft_package_address = ledger.compile_and_publish(this_package!());

    // Instantiate the component
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            nft_package_address,
            "PiersRugyard",
            "instantiate",
            manifest_args!(
                360u64,
                5u64,
                dec!(50),
                owner_resource,
                pool_component,
                early_resource,
                xrd_domain
            ),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    let nft_component = receipt.expect_commit_success().new_component_addresses()[0];
    let nft_resource = receipt.expect_commit_success().new_resource_addresses()[0];

    (
        ledger,
        nft_component,
        nft_resource,
        owner_resource,
        early_resource,
        account,
    )
}

// Creates an environment with minted NFTs and an active auction
pub fn create_prepared_test_environment() -> (
    DefaultLedgerSimulator,
    ComponentAddress,
    ResourceAddress,
    ResourceAddress,
    ResourceAddress,
    Account,
) {
    let (mut ledger, nft_component, nft_resource, owner_resource, early_resource, account) =
        create_test_environment();

    // NFTs
    let mut nfts = vec![];
    for i in 0..80 {
        nfts.push(("https://www.google.com", format!("My NFT {i}")));
    }

    // Mint NFTs and start auction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account.address, owner_resource, dec!(1))
        .call_method(
            nft_component,
            "mint_nfts",
            manifest_args!(nfts),
        )
        .call_method(nft_component, "flip_status", manifest_args!())
        .call_method(nft_component, "start_new_auction", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    receipt.expect_commit_success();

    (
        ledger,
        nft_component,
        nft_resource,
        owner_resource,
        early_resource,
        account,
    )
}

// Gets the balance for a resource in a specific account
pub fn get_account_balance(
    ledger: &mut DefaultLedgerSimulator,
    account: &Account,
    resource: ResourceAddress,
) -> Decimal {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(account.address, "balance", manifest_args!(resource))
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    let balance: Decimal = receipt.expect_commit_success().output(1);

    balance
}

// Gets the currently active auction
pub fn get_current_auction(
    ledger: &mut DefaultLedgerSimulator,
    component: ComponentAddress,
    account: &Account,
) -> Option<Auction> {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "get_current_auction", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    let auction: Option<Auction> = receipt.expect_commit_success().output(1);

    auction
}

// Sets the time on the ledger to the provided time
pub fn change_time(ledger: &mut DefaultLedgerSimulator, timestamp: Instant) {
    let timestamp_ms = timestamp.seconds_since_unix_epoch * 1000;
    let current_round = ledger.get_consensus_manager_state().round;
    let receipt =
        ledger.advance_to_round_at_timestamp(Round::of(current_round.number() + 1), timestamp_ms);
    receipt.expect_commit_success();
}
