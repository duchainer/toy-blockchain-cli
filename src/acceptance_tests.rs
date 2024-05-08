use super::*;

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
    use super::*;

    #[test]
    fn running_with_start_node_keeps_me_running() {
        let node_res = duct::cmd!("cargo", "run", "start_node").start();
        assert!(node_res.is_ok(), "Failed to run: {:?}", node_res);
        if let Ok(node) = node_res {
            sleep(Duration::from_secs(2));

            // assert that it is still running
            if let Ok(val) = node.try_wait() {
                assert_eq!(val, None);
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
            sleep(Duration::from_millis(1000));

            // assert that it is done
            if let Ok(val) = node.try_wait() {
                if val.is_none() {
                    assert!(val.is_some());
                    let _ = node.kill().is_ok();
                };
            }
        }
    }

    #[test]
    fn every_ten_seconds_start_node_should_create_a_block() {
        let block_time_diff = 2;
        let node_res = duct::cmd!("cargo", "run", "start_node", "--block-time", block_time_diff.to_string()).reader();
        let reader = node_res.unwrap();
        // reader.kill().unwrap();

        // As inspired by https://stackoverflow.com/a/31577297/7243716
        let mut buf_reader = BufReader::new(reader);
        let mut output = String::new();

        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 0");
        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 1");
        buf_reader.read_line(&mut output).unwrap();
        assert_contains!(output, "block 2");
        assert!(buf_reader.into_inner().kill().is_ok());


        fn extract_integer_timestamp(line: &str) -> u128 {
            line.split(" ").take(1).collect::<String>()
                .chars().filter(|c| c.is_digit(10))
                .collect::<String>().parse::<u128>().unwrap()
        }
        output.lines().map(extract_integer_timestamp).collect::<Vec<_>>().windows(2).for_each(
            |pair|
                assert_eq!(pair[0] + block_time_diff, pair[1])
        );
    }

    #[test]
    fn account_creation_and_balance() {
        let node_res = duct::cmd!("cargo", "run", "start_node").reader();

    }
}
