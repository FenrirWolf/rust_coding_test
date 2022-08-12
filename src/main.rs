use clap::Parser;
use csv::{ReaderBuilder, Trim, Writer};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnError};

use std::collections::{btree_map::Entry, BTreeMap};
use std::path::{Path, PathBuf};

type ClientId = u16;
type TransactionId = u32;

// The main state of the program. Contains each client's current balance along with a record of
// each Deposit transaction that was processed.
#[derive(Default)]
struct State {
    clients: BTreeMap<ClientId, Client>,
    transactions: BTreeMap<TransactionId, Transaction>,
}

#[derive(Default)]
struct Client {
    available: Decimal,
    held: Decimal,
    locked: bool,
}

#[derive(Default)]
struct Transaction {
    amount: Decimal,
    disputed: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Parser)]
struct Cli {
    file_name: PathBuf,
}

#[serde_as]
#[derive(Deserialize)]
struct InputRow {
    #[serde(rename = "type")]
    kind: TransactionKind,
    #[serde(rename = "client")]
    client_id: ClientId,
    #[serde(rename = "tx")]
    txn_id: TransactionId,
    #[serde_as(deserialize_as = "DefaultOnError")]
    #[serde(default)]
    amount: Decimal,
}

#[derive(Serialize)]
struct OutputRow {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let state = process_input(&cli.file_name)?;

    write_output(state)?;

    Ok(())
}

/// This function reads the input csv file and then builds up a list of clients and their balances
/// from the incoming transaction stream
fn process_input(file_name: &Path) -> anyhow::Result<State> {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(Trim::All)
        .from_path(file_name)?;

    let mut state = State::default();

    for row in reader.deserialize() {
        let input: InputRow = row?;

        handle_transaction(&mut state, input)?;
    }

    Ok(state)
}

/// Handles the five different kinds of transactions and updates the State accordingly
fn handle_transaction(state: &mut State, input: InputRow) -> anyhow::Result<()> {
    // Create or get a client as soon as one is referenced in the transaction stream
    let client = state.clients.entry(input.client_id).or_default();

    // Defer creation of a transaction record until a Deposit occurs
    let txn_entry = state.transactions.entry(input.txn_id);

    match input.kind {
        TransactionKind::Deposit => {
            if !client.locked {
                client.available += input.amount;

                txn_entry.or_default().amount = input.amount;
            }
        }
        TransactionKind::Withdrawal => {
            if !client.locked {
                let diff = client.available - input.amount;

                if diff.is_sign_negative() {
                    return Ok(());
                }

                client.available = diff;
            }
        }
        TransactionKind::Dispute => {
            if let Entry::Occupied(mut entry) = txn_entry {
                let transaction = entry.get_mut();

                if !client.locked && !transaction.disputed {
                    client.held += transaction.amount;
                    client.available -= transaction.amount;

                    transaction.disputed = true;
                }
            }
        }
        TransactionKind::Resolve => {
            if let Entry::Occupied(mut entry) = txn_entry {
                let transaction = entry.get_mut();

                if !client.locked && transaction.disputed {
                    client.held -= transaction.amount;
                    client.available += transaction.amount;

                    transaction.disputed = false;
                }
            }
        }
        TransactionKind::Chargeback => {
            if let Entry::Occupied(mut entry) = txn_entry {
                let transaction = entry.get_mut();

                if !client.locked && transaction.disputed {
                    client.held -= transaction.amount;

                    client.locked = true;
                }
            }
        }
    }

    Ok(())
}

/// Writes program output to stdout
fn write_output(state: State) -> anyhow::Result<()> {
    let mut writer = Writer::from_writer(std::io::stdout());

    for (client_id, client) in state.clients {
        let output = OutputRow {
            client: client_id,
            available: client.available,
            held: client.held,
            total: client.available + client.held,
            locked: client.locked,
        };

        writer.serialize(output)?;
    }

    Ok(())
}
