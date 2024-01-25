// const BLOCK_NUMBER_THRESHOLD: u64 = 100; // Block with height difference more than this will be considered as old and won't be processed
// async fn _execute_chain_monitoring() -> Result<()> {
//     if Timer::is_active() {
//         return Ok(());
//     }

//     log!(
//         "Watch blocks started. {:#?}",
//         get_state!(last_checked_block_height)
//     );

//     let w3 = Web3::new(
//         ICHttp::new(
//             "https://goerli.infura.io/v3/8e4147cd4995430182a09781136f8745",
//             None,
//         )
//         .unwrap(),
//     );

//     let val = w3
//         .eth()
//         .block(BlockId::Number(BlockNumber::Latest), CallOptions::default())
//         .await;

//     let blocks = vec![val.unwrap().unwrap()]; // TODO: Remove unwrap
//     let last_block_height = blocks[0].number.unwrap().0[0];

//     if let Some(last_checked_block_height) = get_state!(last_checked_block_height) {
//         if last_block_height - last_checked_block_height > BLOCK_NUMBER_THRESHOLD {
//             update_state!(last_checked_block_height, Some(last_block_height));
//         }

//         for i in (last_checked_block_height + 1..last_block_height + 1).rev() {
//             let block = w3
//                 .eth()
//                 .block(BlockId::Number(i.into()), CallOptions::default())
//                 .await
//                 .unwrap()
//                 .unwrap(); // TODO: Remove unwrap
//             let block_height = block.number.unwrap().0[0];

//             update_state!(last_checked_block_height, Some(block_height));

//             log!("Block: {:#?}", block.number);
//         }
//     } else {
//         update_state!(last_checked_block_height, Some(last_block_height));
//     }

//     let timer_id = set_timer(Duration::from_secs(get_state!(timer_frequency)), execute);
//     Timer::update(timer_id)?;

//     log!("Watch blocks finished");
//     Ok(())
// }
