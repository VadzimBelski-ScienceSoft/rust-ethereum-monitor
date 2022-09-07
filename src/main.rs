extern crate serde_json;
extern crate ticker;

use std::time::Duration;

use web3::{
    types::{ TransactionId,BlockId},
};

use ticker::Ticker;

#[tokio::main]
async fn main() -> web3::Result {
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

async fn consume(block_id : web3::types::BlockId, web3http : &web3::Web3<web3::transports::Http>) {

    let block = web3http.eth().block(block_id).await;
    let block_object = block.as_ref().unwrap();

    println!("Block: {:?}", block_object);
}

async fn scan_transaction( tx: web3::types::H256, web3http: &web3::Web3<web3::transports::Http>){

    let tr = web3http.eth().transaction(TransactionId::Hash(tx)).await.unwrap();

    let transaction = tr.as_ref().unwrap();

    println!("Addres From: {:?}", transaction.from);
    println!("Addres to: {:?}", transaction.to);
}