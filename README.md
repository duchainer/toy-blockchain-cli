# toy-blockchain-cli

## Run tests
For now, you'll want to run tests using `cargo test -- --test-threads 1`
Because we don't want other tests to send requests to the other tests start_node,
as we have the start_node address hardcoded, and only the first one will run.

## 