# Rust Coding Test

This is a toy payments program I wrote as part of the hiring process for [REDACTED]. It reads transaction data from a CSV file via the command line, processes those transactions for various clients, then writes client balances and other account info to stdout. 

# Building and running

The program is written in Rust. Install Rust and Cargo on your system and then run it like so:

`$ cargo run -- test_data/transactions.csv`

A few different CSV files are included in the `test_data` folder that exercise different edge cases in the code. The program should be able to parse CSV files with any number of whitespaces between values.

The program conforms to the design document as best as I can tell, but I do have a few unresolved questions. Specifically:

* What operations are permitted on locked accounts? There's no information about that in the test document so I'm assuming that none are.

* Can you do chargebacks on withdrawals as well as deposits? If so, how would that behave? The docs only seem to refer to the case of deposits, so I opted to process chargebacks for deposits only.

* On a related note, should a successful chargeback also result in the sender account being credited the amount that was previously withdrawn? I'm guessing no since the test doc never mentions anything of the sort, but that seems like a strange omission.