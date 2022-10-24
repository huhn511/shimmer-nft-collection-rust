// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example mint_issuer_nft --release
// In this example we will mint the issuer nft.
// Rename `.env.example` to `.env` and run 01_create_wallet.rs before

use std::env;

use dotenv::dotenv;
use iota_wallet::{account_manager::AccountManager, ClientOptions, NftOptions, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Collection {
    standard: String,
    version: String,
    r#type: String,
    uri: String,
    name: String,
    description: String,
    issuerName: String,
    collectionId: String,
    collectionName: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CollectionNFT {
    standard: String,
    version: String,
    r#type: String,
    uri: String,
    description: String,
    issuerName: String,
    collectionName: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    // JSON COLLECTION STUFF
    let json_file_path = Path::new("collection.json");
    // let file = File::open(json_file_path);
    // let mut file = File::open(&json_file_path).expect("Error opening File");
    let mut file = File::open(&json_file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // let collection:Collection = serde_json::from_reader(file).expect("e");
    let collection: CollectionNFT = serde_json::from_str(&data).expect("JSON was not well-formatted");

    println!("{:?}", collection);
    // JSON COLLECTION STUFF END

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;
    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;
    account.sync(None).await?;
    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;
    let nft_options = vec![NftOptions {
        address: None,
        immutable_metadata: Some(
            serde_json::to_string(&collection)
                .unwrap()
                .as_bytes()
                .to_vec(),
        ),
        issuer: None,
        metadata: None,
        sender: None,
        tag: None,
    }];

    let transaction = account.mint_nfts(nft_options, None).await?;

    println!(
        "https://explorer.shimmer.network/testnet/transaction/{}.",
        transaction.transaction_id,
    );

    Ok(())
}
