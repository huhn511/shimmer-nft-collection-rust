// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! cargo run --example mint_collection_nft --release
// In this example we will mint the 150 nfts with issuer feature.
// Rename `.env.example` to `.env` and run 01_create_wallet.rs before

use std::{env, str::FromStr};

use dotenv::dotenv;
use iota_client::block::{
    address::{Address, NftAddress},
    output::NftId,
};
use iota_wallet::{account_manager::AccountManager, NftOptions, Result};
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

#[tokio::main]
async fn main() -> Result<()> {
    // This example uses dotenv, which is not safe for use in production
    dotenv().ok();

    let nft_collection_size = 10;
    // Set this to the NFT id from the mint_issuer_nft example
    let issuer_nft_id =
        NftId::from_str("0x9f94fd69519ccda1cc81368b48b91a78582f8fb236531cb049529598cb01c091")
            .unwrap();

    // Create the account manager
    let manager = AccountManager::builder().finish().await?;

    // Get the account we generated with `01_create_wallet`
    let account = manager.get_account("Alice").await?;

    account.sync(None).await?;

    // Set the stronghold password
    manager
        .set_stronghold_password(&env::var("STRONGHOLD_PASSWORD").unwrap())
        .await?;

    let bech32_hrp = account.client().get_bech32_hrp()?;
    let mut nft_options = Vec::new();

    // JSON COLLECTION STUFF
    let json_file_path = Path::new("collection.json");
    // let file = File::open(json_file_path);
    // let mut file = File::open(&json_file_path).expect("Error opening File");
    let mut file = File::open(&json_file_path).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // let collection:Collection = serde_json::from_reader(file).expect("e");
    let mut collection: Collection = serde_json::from_str(&data).expect("JSON was not well-formatted");

    println!("{:?}", collection);
    // JSON COLLECTION STUFF END
    
    // Create the metadata with another index for each
    for index in 0..nft_collection_size {
        // collection.uri = format!("{}/shimmer-{}", collection.uri, index);
        // println!("{:?}", collection.uri);
        // collection.name = format!("{} #{}", collection.name, index);
        // println!("{:?}", collection.name);
        nft_options.push(NftOptions {
            address: None,
            // immutable_metadata: Some(format!("{{\"standard\":\"IRC27\",\"version\":\"v1.0\",\"type\":\"image/png\",\"uri\":\"https://robohash.org/shimmer-#{index}.png\",\"name\":\"Shimmer Test Robot #{index}\",\"description\":\"The Shimmer Test Robot NFT Collection. Robots lovingly delivered by Robohash.org.\",\"issuerName\":\"Alice\",\"collectionId\":\"{issuer_nft_id}\",\"collectionName\":\"Shimmer Test Robot\" }}").as_bytes().to_vec()),
            // immutable_metadata: Some(
            //     serde_json::to_string(&collection)
            //         .unwrap()
            //         .as_bytes()
            //         .to_vec(),
            // ),
            immutable_metadata: None,
            // The NFT address from the NFT we minted in mint_issuer_nft example
            issuer: Some(
                Address::Nft(NftAddress::new(issuer_nft_id)).to_bech32(bech32_hrp.clone()),
            ),
            metadata: None,
            sender: None,
            tag: None,
        });
    }

    // Mint nfts in chunks, since the transaction size is limited
    for nfts in nft_options.chunks(10) {
        let transaction = account.mint_nfts(nfts.to_vec(), None).await?;

        println!(
            "https://explorer.shimmer.network/testnet/transaction/{}.",
            transaction.transaction_id,
        );
        if let Some(block_id) = transaction.block_id {
            println!(
                "Block sent: {}/api/core/v2/blocks/{}",
                &env::var("NODE_URL").unwrap(),
                block_id
            );
            // Try to get the transaction confirmed
            account.retry_until_included(&block_id, None, None).await?;
        }
        // Sync so the new outputs are available again for new transactions
        account.sync(None).await?;
    }

    // After the NFTs are minted, the issuer nft can be burned, to prevent minting any further NFTs in the future

    Ok(())
}
