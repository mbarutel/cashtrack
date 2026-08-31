use std::{path::Path, str::FromStr, time::Instant};

use crate::{
    MyResult, State,
    config::CategoryRule,
    models::{Direction, Transaction},
};

// TODO: This can be done in batches eventually.
// Take the (String, String, String)
// Generate the Transaction Object
// Insert into Database

pub fn import(state: &mut State, path: &Path) -> MyResult<()> {
    let start = Instant::now();

    let mut transactions = read_transactions(path, state.config.rules())?;
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

fn read_transactions(path: &Path, rules: &Vec<CategoryRule>) -> MyResult<Vec<Transaction>> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;

    let mut transactions: Vec<Transaction> = Vec::new();

    for result in reader.deserialize() {
        let (date, amount, description): (String, String, String) = result?;
        let date = chrono::NaiveDate::parse_from_str(&date, "%d/%m/%Y")?;
        let amount = rust_decimal::Decimal::from_str(&amount)?;

        transactions.push(Transaction {
            date,
            direction: Direction::from(amount),
            amount,
            category: categorizer(rules, &description),
            description,
            bank: "Commonwealth".to_string(),
        });
    }

    Ok(transactions)
}

fn categorizer(rules: &Vec<CategoryRule>, description: &String) -> String {
    let mut result = "Unknown".to_string();
    let mut last_prio_level = 0;

    for rule in rules {
        for keyword in rule.keywords() {
            if description
                .to_lowercase()
                .contains(keyword.to_lowercase().as_str())
                && rule.priority() > last_prio_level
            {
                result = rule.subcategory().to_string();
                last_prio_level = rule.priority();
            }
        }
    }

    result
}

fn sort_by_category(transactions: &mut Vec<Transaction>) {
    transactions.sort_by(|a, b| a.category.cmp(&b.category));
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
