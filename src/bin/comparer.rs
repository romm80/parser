mod common;

use clap::Parser;
use parser;
use parser::Transaction;
use std::collections::{HashMap, HashSet};
use std::fs::File;

fn main() {
    let args = common::args::Compare::parse();

    let f1 = File::open(&args.file1).unwrap();
    let f2 = File::open(&args.file2).unwrap();
    match parser::read(f1, args.file1_format.into()) {
        Ok(tx1) => match parser::read(f2, args.file2_format.into()) {
            Ok(tx2) => match compare(tx1, tx2, args.file1, args.file2) {
                Some(diff) => diff.iter().for_each(|d| println!("{}", d)),
                None => {
                    println!("transaction are identical");
                }
            },
            Err(e) => eprintln!("parse file '{}' error {}", args.file2, e),
        },
        Err(e) => eprintln!("parse file '{}' error {}", args.file1, e),
    }
}

fn compare(
    tx1: Vec<Transaction>,
    tx2: Vec<Transaction>,
    f1: String,
    f2: String,
) -> Option<Vec<String>> {
    let tx1_map: HashMap<u64, Transaction> = tx1.into_iter().map(|tx| (tx.tx_id, tx)).collect();
    let tx2_map: HashMap<u64, Transaction> = tx2.into_iter().map(|tx| (tx.tx_id, tx)).collect();

    let mut tx_ids = HashSet::with_capacity(tx1_map.len() + tx2_map.len());
    tx_ids.extend(tx1_map.iter().map(|tx| tx.0));
    tx_ids.extend(tx2_map.iter().map(|tx| tx.0));
    let mut diff = Vec::new();
    tx_ids
        .iter()
        .for_each(|id| match (tx1_map.get(id), tx2_map.get(id)) {
            (Some(tx), None) => diff.push(format!(
                "transaction id:{} in file '{}' not found in file '{}'",
                tx.tx_id, f1, f2
            )),
            (None, Some(tx)) => diff.push(format!(
                "transaction id:{} in file '{}' not found in file '{}'",
                tx.tx_id, f2, f1
            )),
            (Some(tx1), Some(tx2)) => {
                if tx1 != tx2 {
                    diff.push(format!(
                        "transaction id:{} in file '{}' not equal transaction id:{} in file '{}'",
                        tx1.tx_id, f1, tx2.tx_id, f2
                    ))
                }
            }
            (None, None) => {}
        });
    if diff.is_empty() {
        return None;
    }
    Some(diff)
}
