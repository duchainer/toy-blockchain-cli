
/*
Voici notre test Rust:

Blockchain Simulation in Rust

In this assignment, we will build a toy blockchain called ‘B’.

Like other blockchains, B creates new blocks. Therefore, when we send a transaction command, it takes a few seconds to be confirmed because the transaction needs to be included in a new block. As on some real blockchains, B creates new blocks at regular time intervals of 10 seconds. So, let’s say blocks are minted at T=10, T=20, T=30, etc. If we send a transaction a T=7, we will wait 3 seconds for its confirmation. If we send one at T=12, we will wait 8 seconds for the transaction to be confirmed in a new block.

There are two types of transactions on B, one for creating accounts and the other for transferring funds.

There is also a read command for viewing an account balance. However, it is a read command, not a transaction. So the balance command should instantaneously show the result.

Here are its desired features:

#1

b start-node

The `start-node` command starts a local, new B blockchain server. Keep it running in a separate terminal. It should stop with Ctrl-C.

#2

b create-account <id-of-account> <starting-balance>

The `create-account` transaction should create an account on B.

#3

b transfer <from-account> <to-account> <amount>

The `transfer` transaction should send funds from one account to another on B.

#4

b balance <account>

The `balance` command should display the funds of a B account. Remember, this is a read command.


Miscellaneous:

Display meaningful error messages only if the user misuses a command. You do not have to handle other errors.

The B simulation is a local, single-threaded CLI. There is no need for cryptography! Account information is not permanently stored, as the `start-node` command will start a new blockchain.

As long as the four commands work as expected, there is no single “right” way of doing this simulation project 🙂

Cheers
*/
#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read};
    use std::thread::sleep;
    use std::time::Duration;

    use assertables::assert_contains;
    use assertables::assert_contains_as_result;
    use std::string::String;
    use super::*;

    #[test]
    fn running_with_start_node_keeps_me_running() {
        let minimum_living_time = 2;
        let node_res = duct::cmd!("cargo", "run", "start_node").start();
        assert!(node_res.is_ok(), "Failed to run: {:?}", node_res);
        if let Ok(node) = node_res {
            sleep(Duration::from_secs(minimum_living_time));

            // assert that it is still running
            if let Ok(val) = node.try_wait() {
                assert!(val.is_none(), "The node stopped running under {} seconds", minimum_living_time);
            }

            // cleanup
            assert!(node.kill().is_ok());
        }
    }

    #[test]
    fn when_not_using_the_start_node_command_be_short_lived() {
        // This should be running help
        let node_res = duct::cmd!("cargo", "run").start();
        assert!(node_res.is_ok(), "Failed to run: {:?}", node_res);
        if let Ok(node) = node_res {
            sleep(Duration::from_millis(1500));

            if let Ok(val) = node.try_wait() {
                if val.is_none() {
                    assert!(val.is_some(), "It was not done yet?");
                    let _ = node.kill().is_ok();
                };
            }
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
        // reader.kill().unwrap();

        // As inspired by https://stackoverflow.com/a/31577297/7243716
        let mut buf_reader = BufReader::new(reader);
        let mut output = String::new();

        // sleep(Duration::from_secs(block_time_diff));
        // // Throwing away the "listening started, ready to accept\n"
        // buf_reader.read_line(&mut output).unwrap();
        // output = String::new();

        sleep(Duration::from_secs(block_time_diff));
        sleep(Duration::from_secs(block_time_diff));
        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 0");

        sleep(Duration::from_secs(block_time_diff));
        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 1");

        sleep(Duration::from_secs(block_time_diff));
        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 2");
        assert!(buf_reader.into_inner().kill().is_ok());


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

        let account_creation_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .read().expect("The create_account command should work");

        assert_contains!(account_creation_output , "Created account");
        sleep(Duration::from_secs(block_time));

        let account_creation2_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .read().expect("The create_account command should work");

        assert!(node_handle.kill().is_ok());
        assert_contains!(account_creation2_output , "Already existing account");
    }
    #[test]
    #[ntest::timeout(5000)] //In case the client are blocking in some way, we rather abort the test than wait.
    fn account_creation_and_already_being_created() {
        let block_time = 1;
        let balance: u128 = 1000;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time.to_string()).start();
        let node_handle = node_res.expect("The start_node command should work");

        let account_creation_output = duct::cmd!("cargo", "run", "create_account", "bob", balance.to_string())
            .start().expect("The create_account command should work");
        sleep(Duration::from_secs(block_time));
        if account_creation_output.kill().is_ok() {println!("Should not be long running")}

        // TODO Add good error message printed when giving too much arguments to it
        // let balance_output = duct::cmd!("cargo", "run", "balance", "bob", balance.to_string())
        //     .read().;

        // let balance_output = duct::cmd!("cargo", "run", "balance", "bob")
        //     .read().expect("The balance command should work");
        //
        // assert_contains!(balance_output, "No account");

        sleep(Duration::from_secs(block_time));

        let balance_output = duct::cmd!("cargo", "run", "balance", "bob")
            .read().expect("The balance command should work");

        sleep(Duration::from_secs(block_time));
        if account_creation_output.kill().is_ok() {println!("Should not be long running")}

        assert!(node_handle.kill().is_ok());
        assert_contains!(balance_output, &balance.to_string());
    }
}
