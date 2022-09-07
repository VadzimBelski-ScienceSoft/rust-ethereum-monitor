extern crate serde_json;
extern crate ticker;
extern crate bitcoin;
extern crate hex;

use web3::signing::keccak256;
use std::time::Duration;

use web3::{
    types::{ TransactionId, BlockId},
};

use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1};
use web3::types::Address;
use std::str::FromStr;


use bip0039::{Count, Mnemonic};
use bitcoin::secp256k1::ffi::types::AlignedType;
use bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, ExtendedPubKey};
use bitcoin::PublicKey;

use ticker::Ticker;

#[tokio::main]
async fn main() -> web3::Result {

    generate_keypair();
    // Sign up at infura > choose the desired network (eg Rinkeby) > copy the endpoint url into the below
    // If you need test ether use a faucet, eg https://faucet.rinkeby.io/
    let transport = web3::transports::Http::new("https://ropsten.infura.io/v3/f1a6a5d57420473b975975c55f5d3666")?;
    let web3http = web3::Web3::new(transport);

    let ws = web3::transports::WebSocket::new("wss://ropsten.infura.io/ws/v3/f1a6a5d57420473b975975c55f5d3666").await?;
    let web3 = web3::Web3::new(ws.clone());

    let mut sub = web3.eth_subscribe().subscribe_new_heads().await?;

    println!("Got subscription id: {:?}", sub.id());

    // future::ready(&mut sub).then(|x| async move{
            
    //         println!("Got: {:?}", x);

    //         // let result = x;

    //         // let block = web3.eth().block(BlockId::from(result.number.unwrap())).await.unwrap();
    //         // let block_object = block.as_ref().unwrap();

    //         future::ready(())
    //     });


    // while let Some(x) = sub.next().await {
    //     consume(BlockId::from(x.unwrap().number.unwrap()), &web3http).await;
    // }

    // sub.unsubscribe().await?;


    let current_block_number = web3.eth().block_number().await;

    let mut next_block_number = current_block_number.unwrap();
    let mut latest_scanned_block = web3::types::U64::from(0);

    let ticker = Ticker::new(0.., Duration::from_secs(5));

    for _ in ticker {
 
        println!("We are on the block {:?}", next_block_number);

        let block = web3.eth().block(BlockId::from(next_block_number)).await.unwrap();

        println!("Block details are {:?}", block);

        if block != None {

            let block_object = block.as_ref().unwrap();
        
            if latest_scanned_block != block_object.number.unwrap() {
                for tx in &block_object.transactions {
                    scan_transaction(*tx, &web3http).await;
                    println!("Transaction {:?}", tx);
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

// async fn consume(block_id : web3::types::BlockId, web3http : &web3::Web3<web3::transports::Http>) {

//     let block = web3http.eth().block(block_id).await;
//     let block_object = block.as_ref().unwrap();

//     println!("Block: {:?}", block_object);
// }

async fn scan_transaction( tx: web3::types::H256, web3http: &web3::Web3<web3::transports::Http>){

    let tr = web3http.eth().transaction(TransactionId::Hash(tx)).await.unwrap();

    let transaction = tr.as_ref().unwrap();

    println!("Addres From: {:?}", transaction.from);
    println!("Addres to: {:?}", transaction.to);
}

fn generate_keypair() {
    
    let network = bitcoin::Network::Bitcoin;

    // Generates an English mnemonic with 12 words randomly
    //let mnemonic = Mnemonic::generate(Count::Words12);
    let mnemonic = Mnemonic::from_phrase("jealous picnic lazy lend basic kangaroo debate inspire select brisk neither license").unwrap();
    // Gets the phrase
    let _phrase = mnemonic.phrase();

    println!("Phrase generated: {}", _phrase);

    // Generates the HD wallet seed from the mnemonic and the passphrase.
    let seed = mnemonic.to_seed("");

    // we need secp256k1 context for key derivation
    let mut buf: Vec<AlignedType> = Vec::new();
    buf.resize(Secp256k1::preallocate_size(), AlignedType::zeroed());
    let secp = Secp256k1::preallocated_new(buf.as_mut_slice()).unwrap();

    // calculate root key from seed
    let root = ExtendedPrivKey::new_master(network, &seed).unwrap();
    println!("Root key: {}", root);

    println!("Root hex: {}", hex::encode(root.to_priv().to_bytes()));

    // derive child xpub
    let path = DerivationPath::from_str("m/44'/60'/0'/0/0").unwrap();
    let child = root.derive_priv(&secp, &path).unwrap();
    println!("Child at {}: {}", path, child);

    println!("Child private hex: {}", hex::encode(child.to_priv().to_bytes()));


    let xpub = ExtendedPubKey::from_priv(&secp, &child);
    println!("Public key at {}: {}", path, xpub);


    let public_key = xpub.public_key;

    let public_key = public_key.serialize_uncompressed();
    let hash = keccak256(&public_key[1..]);

    let address = Address::from_slice(&hash[12..]);

    println!("address: {:?}", &address);
}