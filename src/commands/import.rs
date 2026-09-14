use std::{path::Path, time::Instant};

use crate::{
    config::Category,
    models::{Bank, Direction, Transaction},
    MyResult, State,
};

// TODO: This can be done in batches eventually.
// Take the (String, String, String)
// Generate the Transaction Object
// Insert into Database

pub fn import(state: &mut State, path: &Path, bank: Bank) -> MyResult<()> {
    let start = Instant::now();

    let mut transactions = read_transactions(state.config.categories(), path, bank)?;
    sort_by_date(&mut transactions);

    let inserted_count = state.db.insert_transactions(&transactions)?;

    let elapsed = start.elapsed();
    println!(
        "Imported {} Complete: {}ms",
        inserted_count,
        elapsed.as_millis()
    );
    Ok(())
}

fn read_transactions(rules: &Vec<Category>, path: &Path, bank: Bank) -> MyResult<Vec<Transaction>> {
    Ok(bank
        .parse(path)?
        .into_iter()
        .map(|row| {
            let transaction = Transaction {
                date: row.date,
                direction: Direction::from(row.amount),
                amount: row.amount.abs(),
                category: categorizer(rules, &row.description),
                description: row.description,
                bank: bank.to_string(),
            };

            println!("{:#?}", transaction);

            transaction
        })
        .collect())
}

fn categorizer(categories: &Vec<Category>, description: &String) -> String {
    let mut result = "Unknown".to_string();
    let mut last_prio_level = 0;

    for category in categories {
        for keyword in category.keywords() {
            if description
                .to_lowercase()
                .contains(keyword.to_lowercase().as_str())
                && category.priority() > last_prio_level
            {
                result = category.title().to_string();
                last_prio_level = category.priority();
            }
        }
    }

    result
}

fn sort_by_date(transactions: &mut Vec<Transaction>) {
    transactions.sort_by(|a, b| a.date.cmp(&b.date));
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn mock_rules() -> Vec<CategoryRule> {
//         vec![
//             CategoryRule {
//                 subcategory: "Groceries".to_string(),
//                 keywords: vec![
//                     "woolworths".to_string(),
//                     "coles".to_string(),
//                     "pearl".to_string(),
//                 ],
//                 priority: 60,
//             },
//             CategoryRule {
//                 subcategory: "Fuel".to_string(),
//                 keywords: vec!["ampol".to_string(), "pearl".to_string()],
//                 priority: 70,
//             },
//         ]
//     }

//     #[test]
//     fn assigns_category_on_keyword_match() {
//         let rules = mock_rules();
//         let description =
//             "WOOLWORTHS 2764 NERANG QL AUS Card xx8935 Value Date: 31/07/2026".to_string();
//         let result = categorizer(&rules, &description);
//         assert_eq!(result, "Groceries")
//     }

//     #[test]
//     fn assigns_higher_priority_on_multiple_keyword_match() {
//         let rules = mock_rules();
//         let description =
//             "PEARL SOUTHPORT SOUTHPORT QL AUS Card xx5231 Value Date: 29/07/2026".to_string();
//         let result = categorizer(&rules, &description);
//         assert_eq!(result, "Fuel")
//     }

//     #[test]
//     fn returns_unknown_when_no_match() {
//         let rules = mock_rules();
//         let description = "EasyPark".to_string();
//         let result = categorizer(&rules, &description);
//         assert_eq!(result, "Unknown")
//     }
// }
