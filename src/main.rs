use anyhow::Result;
mod eth_wallet;
mod utils;
use std::env;
use ticker::Ticker;
use web3::{
    types::{ TransactionId, BlockId},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let endpoint = env::var("INFURA_WS")?;
    let mnemonic_phrase = env::var("MNEMONIC_PHRASE")?;
    let wallet_file_path = "crypto_wallet.json";

    if endpoint.is_empty() {
        return Err(anyhow::anyhow!("Missing endpoint in .env"));
    }

    eth_wallet::Wallet::from_file(wallet_file_path)
        .map_err(|error| {
            println!("In file {}, found error: {}", wallet_file_path, error);

            if mnemonic_phrase.is_empty() {
                let (secret_key, pub_key) = eth_wallet::generate_keypair();

                println!("secret key: {}", &secret_key.display_secret());
                println!("public key: {}", &pub_key.to_string());

                let pub_address = eth_wallet::public_key_address(&pub_key);
                println!("public address: {:?}", pub_address);

                let crypto_wallet = eth_wallet::Wallet::new(&secret_key, &pub_key);
                println!("crypto_wallet: {:?}", &crypto_wallet);

                crypto_wallet
                    .save_to_file(wallet_file_path)
                    .map_err(|err| println!("{:?}", err))
                    .ok();
            } else {
                let (secret_key, pub_key) =
                    eth_wallet::generate_keypair_from_mnemonic_phrase(mnemonic_phrase);

                println!("secret key: {}", &secret_key.display_secret());
                println!("public key: {}", &pub_key.to_string());

                let pub_address = eth_wallet::public_key_address(&pub_key);
                println!("public address: {:?}", pub_address);

                let crypto_wallet = eth_wallet::Wallet::new(&secret_key, &pub_key);
                println!("crypto_wallet: {:?}", &crypto_wallet);

                crypto_wallet
                    .save_to_file(wallet_file_path)
                    .map_err(|err| println!("{:?}", err))
                    .ok();
            }
        })
        .ok();

    let loaded_wallet = eth_wallet::Wallet::from_file(wallet_file_path)?;
    println!("loaded_wallet: {:?}", loaded_wallet);

    let web3_con = eth_wallet::establish_web3_connection(&endpoint).await?;

    let block_number = web3_con.eth().block_number().await?;
    println!("block number: {}", &block_number);

    let balance = loaded_wallet.get_balance_in_eth(&web3_con).await?;
    println!("wallet balance: {} eth", &balance);

    // let transaction =
    //     eth_wallet::create_eth_transaction(Address::from_str("0x4fill in address here")?, 0.01);
    // let transact_hash =
    //     eth_wallet::sign_and_send(&web3_con, transaction, &loaded_wallet.get_secret_key()?).await?;

    // println!("transaction hash: {:?}", transact_hash);


    let mut next_block_number = block_number;
    let mut latest_scanned_block = web3::types::U64::from(0);

    let ticker = Ticker::new(0.., std::time::Duration::from_secs(5));

    for _ in ticker {
 
        println!("\n\n\n------------ We are on the block {:?} ----------------", next_block_number);

        let block = web3_con.eth().block(BlockId::from(next_block_number)).await.unwrap();
        //println!("Block details are {:?}", block);

        if block != None {

            let block_object = block.as_ref().unwrap();
        
            if latest_scanned_block != block_object.number.unwrap() {
                for tx in &block_object.transactions {
                    println!("---Transaction {:?}---", tx);

                    let tr = web3_con.eth().transaction(TransactionId::Hash(*tx)).await.unwrap();
                    let transaction = tr.as_ref().unwrap();

                    println!("Addres From: {:?}", transaction.from);
                    println!("Addres to: {:?}", transaction.to);

                    latest_scanned_block = block_object.number.unwrap();
                }
            }

            let increment = 1 as u64;
            next_block_number = block_object.number.unwrap() + increment;
            println!("Setting next block as {:?}", next_block_number);
            

        }else{
            println!("No more blocks");
        }            
    
    }

    Ok(())
}
