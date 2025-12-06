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
            Ok(tx2) => {
                let diffs = compare(&tx1, &tx2);
                if diffs.is_empty() {
                    println!("transaction are identical");
                    return;
                }
                diffs.into_iter().for_each(|diff| match diff {
                    (Some(tx), None) => {
                        println!(
                            "transaction id:{} in file '{}' not found in file '{}'",
                            tx.tx_id, args.file1, args.file2
                        )
                    }
                    (None, Some(tx)) => {
                        println!(
                            "transaction id:{} in file '{}' not found in file '{}'",
                            tx.tx_id, args.file1, args.file2
                        )
                    }
                    (Some(l), Some(r)) => {
                        println!(
                            "transaction id:{} in file '{}' not equal transaction id:{} in file '{}'",
                            l.tx_id, args.file1, r.tx_id, args.file2
                        )
                    }
                    (_, _) => {}
                })
            }
            Err(e) => eprintln!("parse file '{}' error {}", args.file2, e),
        },
        Err(e) => eprintln!("parse file '{}' error {}", args.file1, e),
    }
}

fn compare<'a>(
    left: &'a [Transaction],
    right: &'a [Transaction],
) -> Vec<(Option<&'a Transaction>, Option<&'a Transaction>)> {
    let tx1_map: HashMap<u64, &Transaction> = left.iter().map(|tx| (tx.tx_id, tx)).collect();
    let tx2_map: HashMap<u64, &Transaction> = right.iter().map(|tx| (tx.tx_id, tx)).collect();

    let mut tx_ids = HashSet::with_capacity(tx1_map.len() + tx2_map.len());
    tx_ids.extend(tx1_map.iter().map(|tx| tx.0));
    tx_ids.extend(tx2_map.iter().map(|tx| tx.0));
    let mut diff = Vec::new();
    tx_ids
        .iter()
        .for_each(|id| match (tx1_map.get(id), tx2_map.get(id)) {
            (Some(&tx), None) => diff.push((Some(tx), None)),
            (None, Some(&tx)) => diff.push((None, Some(tx))),
            (Some(&l), Some(&r)) => {
                if l != r {
                    diff.push((Some(l), Some(r)))
                }
            }
            (None, None) => {}
        });
    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::{Status, TransactionType};

    fn transactions() -> Vec<Transaction> {
        vec![
            Transaction {
                tx_id: 1,
                tx_type: TransactionType::DEPOSIT,
                from_user_id: 12,
                to_user_id: 13,
                amount: 14,
                timestamp: 15,
                status: Status::SUCCESS,
                description: "desc1".to_string(),
            },
            Transaction {
                tx_id: 2,
                tx_type: TransactionType::DEPOSIT,
                from_user_id: 22,
                to_user_id: 33,
                amount: 44,
                timestamp: 55,
                status: Status::FAILURE,
                description: "desc2".to_string(),
            },
        ]
    }

    #[test]
    fn test_identical_compare() {
        let left = transactions();
        let right = transactions();

        assert_eq!(
            compare(&left, &right),
            Vec::<(Option<&Transaction>, Option<&Transaction>)>::new()
        );
    }

    #[test]
    fn test_not_equal_one_of_compare() {
        let mut left = transactions();
        let right = transactions();
        left[0].tx_type = TransactionType::WITHDRAWAL;

        assert_eq!(compare(&left, &right), vec![(Some(&left[0]), Some(&right[0]))]);
    }

    #[test]
    fn test_missing_left_compare() {
        let mut left = transactions();
        let right = transactions();
        left.remove(1);

        assert_eq!(compare(&left, &right), vec![(None, Some(&right[1]))]);
    }

    #[test]
    fn test_missing_right_compare() {
        let left = transactions();
        let mut right = transactions();
        right.remove(1);

        assert_eq!(compare(&left, &right), vec![(Some(&left[1]), None)]);
    }
}
