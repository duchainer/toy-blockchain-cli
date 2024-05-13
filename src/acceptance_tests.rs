#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read};
    use std::string::String;
    use std::thread::sleep;
    use std::time::Duration;

    use assertables::{assert_contains, assert_not_contains};
    use assertables::{assert_contains_as_result, assert_not_contains_as_result};
    use scopeguard::defer;

    #[test]
    fn running_with_start_node_keeps_me_running() {
        let minimum_living_time = 2;
        let node = duct::cmd!("cargo", "run", "start_node").start().expect("We should be able to run start_node");
        defer! {
            // cleanup even when we panic and fail the test.
            assert!(node.kill().is_ok());
         }
        sleep(Duration::from_secs(minimum_living_time));

        // assert that it is still running
        if let Ok(val) = node.try_wait() {
            assert!(val.is_none(), "The node stopped running under {} seconds", minimum_living_time);
        }
    }

    #[test]
    fn when_not_using_the_start_node_command_be_short_lived() {
        // This should be running help
        let node = duct::cmd!("cargo", "run").start().expect("We should be able to run start_node");
        defer! {
            // cleanup even when we panic and fail the test.
            assert!(node.kill().is_ok());
         }
        sleep(Duration::from_millis(1500));
        if let Ok(val) = node.try_wait() {
            if val.is_none() {
                assert!(val.is_some(), "It was not done yet?");
                let _ = node.kill().is_ok();
            };
        }
    }

    // NOTE: HACK: For now, you'll want to run tests using `cargo test -- --test-threads 1`
    // Because we don't want other tests to send requests to the other tests start_node
    // TODO Find a better way
    #[test]
    fn every_n_seconds_start_node_should_create_a_block() {
        let block_time_diff = 2;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time_diff.to_string()).reader();
        let reader = node_res.unwrap();
        // NOTE: A ReaderHandle is killed when dropped, so we don't need any additional cleanup even when we panic and fail the test.
        // According to duct docs: https://docs.rs/duct/latest/duct/struct.ReaderHandle.html

        // As inspired by https://stackoverflow.com/a/31577297/7243716
        let mut buf_reader = BufReader::new(reader);
        let mut output = String::new();

        // sleep(Duration::from_secs(block_time_diff));
        // // Throwing away the "listening started, ready to accept\n"
        // buf_reader.read_line(&mut output).unwrap();
        // output = String::new();

        sleep(Duration::from_secs(block_time_diff));
        sleep(Duration::from_secs(block_time_diff));
        if let Ok(_val) = buf_reader.read_line(&mut output) {
            assert_contains!(output, "current_block_num: 0");
        }

        sleep(Duration::from_secs(block_time_diff));
        if let Ok(_val) = buf_reader.read_line(&mut output) {
            assert_contains!(output, "current_block_num: 1");
        }

        sleep(Duration::from_secs(block_time_diff));
        buf_reader.read_line(&mut output).unwrap();
        if let Ok(_val) = buf_reader.read_line(&mut output) {
            assert_contains!(output, "current_block_num: 2");
        }


        fn extract_integer_timestamp(line: &str) -> u64 {
            line.split(" ").take(1).collect::<String>()
                .chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u64>().unwrap()
        }
        output.lines().map(extract_integer_timestamp).collect::<Vec<_>>().windows(2).for_each(
            |pair|
                assert_eq!(pair[0] + block_time_diff, pair[1])
        );
    }

    #[test]
    fn account_creation_and_balance() {
        let block_time = 1;
        let balance: u128 = 1000;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time.to_string()).start();
        let node_handle = node_res.expect("The start_node command should work");
        defer! {
            // cleanup even when we panic and fail the test.
            assert!(node_handle.kill().is_ok());
        }

        sleep(Duration::from_secs(block_time));
        sleep(Duration::from_secs(block_time));
        let account_creation_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .read().expect("The create_account command should work");

        assert_contains!(account_creation_output , "Created account");
        sleep(Duration::from_secs(block_time));

        let account_creation2_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .read().expect("The create_account command should work");

        assert_contains!(account_creation2_output , "Already existing account");
    }

    #[test]
    fn transactions() {
        let block_time = 2;
        // let balance: u128 = 1000;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time.to_string()).start();
        let node_handle = node_res.expect("The start_node command should work");
        defer! {
            // cleanup even when we panic and fail the test.
            assert!(node_handle.kill().is_ok());
        }

        sleep(Duration::from_secs(block_time));
        sleep(Duration::from_secs(block_time));
        let initial_accounts = [("alice", 1000), ("bob", 9000)];
        let initial_account_names = initial_accounts.map(|(name, _)| name);
        let account_creation_outputs = initial_accounts.map(|account|
            duct::cmd!("cargo", "run", "create_account", account.0, account.1.to_string())
                .read().expect("The create_account command should work"));
        sleep(Duration::from_secs(block_time));

        let transfer_amount = 1000;
        let transaction_output =
            duct::cmd!("cargo", "run", "transfer", initial_account_names[0], initial_account_names[1], transfer_amount.to_string())
                .read().expect("The transfer command should work");

        let balance_output1_before_block =
            duct::cmd!("cargo", "run", "balance", initial_account_names[0])
                .read().expect("The balance command should work");
        let balance_output2_before_block =
            duct::cmd!("cargo", "run", "balance", initial_account_names[1])
                .read().expect("The balance command should work");
        sleep(Duration::from_secs(block_time + 1));

        let balance_output1_after_block =
            duct::cmd!("cargo", "run", "balance", initial_account_names[0])
                .read().expect("The balance command should work");
        let balance_output2_after_block =
            duct::cmd!("cargo", "run", "balance", initial_account_names[1])
                .read().expect("The balance command should work");

        account_creation_outputs.iter().for_each(
            |output| { assert_contains!(output , "Created account"); }
        );
        assert_contains!(transaction_output, &"Will add this transaction in the next block".to_string());
        assert_contains!(&balance_output1_before_block, &format!(" {}",initial_accounts[0].1));
        assert_contains!(&balance_output2_before_block, &format!(" {}",initial_accounts[1].1));
        assert_contains!(&balance_output2_after_block, &format!(" {}",initial_accounts[1].1 + transfer_amount));
        assert_contains!(&balance_output1_after_block, &format!(" {}",initial_accounts[0].1 - transfer_amount));
    }

    #[test]
    //In case the client are blocking in some way, we rather abort the test than wait.
    #[ntest::timeout(5000)]
    fn account_creation_and_already_being_created() {
        let block_time = 1;
        let balance: u128 = 1000;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time.to_string()).start();
        let node_handle = node_res.expect("The start_node command should work");
        defer! {
            // cleanup even when we panic and fail the test.
            assert!(node_handle.kill().is_ok());
        }

        // To be sure that the node is properly started already
        sleep(Duration::from_secs(block_time));

        let balance_output = duct::cmd!("cargo", "run", "balance", "bob")
            .read().expect("The balance command should work");

        let account_creation_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .start().expect("The create_account command should work");

        sleep(Duration::from_secs(block_time));
        if account_creation_output.kill().is_ok() { println!("Should not be long running") }

        let balance_output2 = duct::cmd!("cargo", "run", "balance", "bob")
            .read().expect("The balance command should work");


        //
        // assertions
        //
        assert_contains!(balance_output, "No account found");
        assert_contains!(balance_output2, &balance.to_string());
        assert_not_contains!(balance_output2, &"created".to_string());
        assert_not_contains!(balance_output2, &"Already existing account".to_string());
    }
}