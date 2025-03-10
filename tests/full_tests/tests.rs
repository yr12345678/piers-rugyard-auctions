use crate::full_tests::helpers::*;
use piers_rugyard::types::Auction;
use scrypto_test::prelude::*;

#[test]
fn can_instantiate() {
    let (_, _, _, _, _, _) = create_prepared_test_environment();
}

#[test]
fn perform_complete_auction() {
    // Create a test environment with an active auction
    let (mut ledger, component, nft_resource, owner_resource, early_resource, owner_account) =
        create_prepared_test_environment();
    let account1 = create_account(&mut ledger);
    let account2 = create_account(&mut ledger);
    let account3 = create_account(&mut ledger);

    // Place bid with account 1
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account1.address, XRD, dec!(50))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .call_method_with_name_lookup(component, "bid", |lookup| {
            (lookup.bucket("xrd_bucket"), account1.address)
        })
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account1.public_key)],
    );

    receipt.expect_commit_success();

    // Place bid with account 2
    let account1_balance_old = get_account_balance(&mut ledger, &account1, XRD);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account2.address, XRD, dec!(100))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .call_method_with_name_lookup(component, "bid", |lookup| {
            (lookup.bucket("xrd_bucket"), account2.address)
        })
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account2.public_key)],
    );

    receipt.expect_commit_success();

    // Assert account 1 got their bid back
    let account1_balance_new = get_account_balance(&mut ledger, &account1, XRD);
    assert!(
        account1_balance_new == account1_balance_old + 50,
        "Account 1 balance incorrect"
    );

    // Place bid with account 3
    let account2_balance_old = get_account_balance(&mut ledger, &account2, XRD);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account3.address, XRD, dec!(150))
        .take_all_from_worktop(XRD, "xrd_bucket")
        .call_method_with_name_lookup(component, "bid", |lookup| {
            (lookup.bucket("xrd_bucket"), account3.address)
        })
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account3.public_key)],
    );

    receipt.expect_commit_success();

    // Assert account 2 got their bid back
    let account2_balance_new = get_account_balance(&mut ledger, &account2, XRD);
    assert!(
        account2_balance_new == account2_balance_old + 100,
        "Account 2 balance incorrect"
    );

    // Forward time to end the auction
    let auction: Auction = get_current_auction(&mut ledger, component, &account1).unwrap();
    change_time(&mut ledger, auction.end_timestamp);

    // Settle auction with account 1
    let account1_balance_old = get_account_balance(&mut ledger, &account1, XRD);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "settle_auction",
            manifest_args!(account1.address),
        )
        .deposit_batch(account1.address, ManifestExpression::EntireWorktop)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account1.public_key)],
    );

    receipt.expect_commit_success();

    // Assert account 3 got the NFT
    let account3_nft_balance = get_account_balance(&mut ledger, &account3, nft_resource);
    assert!(account3_nft_balance == dec!(1), "Did not get NFT");

    // Assert account 1 got the settlement reward
    let account1_balance_new = get_account_balance(&mut ledger, &account1, XRD);
    assert!(
        account1_balance_new == account1_balance_old + dec!(7.5),
        "Did not get settlement reward"
    );

    // Check profit
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(component, "get_profit_amount", manifest_args!())
        .deposit_entire_worktop(owner_account.address)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
    );

    let profit_amount: Decimal = receipt.expect_commit_success().output(1);

    // Withdraw profit
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            owner_account.address,
            "create_proof_of_amount",
            manifest_args!(owner_resource, dec!(1)),
        )
        .call_method(component, "withdraw_profits", manifest_args!())
        .deposit_entire_worktop(owner_account.address)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(
            &owner_account.public_key,
        )],
    );

    receipt.expect_commit_success();
    let owner_account_balance = get_account_balance(&mut ledger, &owner_account, early_resource);

    assert!(
        owner_account_balance == profit_amount,
        "Did not withdraw profit succesfully"
    );
}

#[test]
fn perform_massive_auction_cost_test() {
    // Create a test environment with an active auction
    let (mut ledger, component, nft_resource, owner_resource, early_resource, owner_account) =
        create_prepared_test_environment();
    let account1 = create_account(&mut ledger);
    let account2 = create_account(&mut ledger);
    let account3 = create_account(&mut ledger);

    // Place bids with account 2
    get_shitton_of_xrd(&mut ledger, &account2);
    for _ in 0..80 {
        complete_auction_process(&mut ledger, &account2, component);
    }
}

fn complete_auction_process(ledger: &mut DefaultLedgerSimulator, account: &crate::full_tests::helpers::Account, component: ComponentAddress) {
    for i in 1..10 {
        let bid_amount = i * 50;
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(account.address, XRD, Decimal::from(bid_amount))
            .take_all_from_worktop(XRD, "xrd_bucket")
            .call_method_with_name_lookup(component, "bid", |lookup| {
                (lookup.bucket("xrd_bucket"), account.address)
            })
            .build();

        let receipt = ledger.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        let cost = receipt.fee_summary.total_cost();
        println!("Bid {} cost {}", i, cost);    
        let auction: Auction = get_current_auction(ledger, component, account).unwrap();
        println!("{:?}", auction.bid_history)
    }

    // Forward time to end the auction
    let auction: Auction = get_current_auction(ledger, component, account).unwrap();
    println!("Bid count: {}", auction.bid_count);
    change_time(ledger, auction.end_timestamp);

    // Settle auction with account 1
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component,
            "settle_auction",
            manifest_args!(account.address),
        )
        .deposit_batch(account.address, ManifestExpression::EntireWorktop)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
    );

    receipt.expect_commit_success();
    let cost = receipt.fee_summary.total_cost();
    println!("settle cost {}", cost);  
}
