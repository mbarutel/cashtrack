use std::{fs, path::Path, str::FromStr};

use crate::{
    MyResult,
    db::Database,
    models::{CategoryRule, Config, Transaction},
};

pub fn import(path: &Path) -> MyResult<()> {
    let config = read_categories(Path::new("./categories.yaml"))?;
    let transactions = read_transactions(path, &config.rules)?;

    let mut db = Database::new(Path::new("cashtrack.db"))?;
    db.insert_transactions(&transactions)?;

    print_transactions(transactions);

    Ok(())
}

fn print_transactions(transactions: Vec<Transaction>) {
    let amount_width = transactions
        .iter()
        .map(|t| format!("{:.2}", t.amount).len())
        .max()
        .unwrap_or(0);
    let category_width = transactions
        .iter()
        .map(|t| t.category.len())
        .max()
        .unwrap_or(0);

    for transaction in transactions {
        println!(
            "{} | {:>amount_width$.2} | {:>category_width$} | {}",
            transaction.date, transaction.amount, transaction.category, transaction.description
        );
    }
}

fn read_categories(path: &Path) -> MyResult<Config> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {}", path.display(), err))?;
    let config = serde_yaml::from_str(&contents)
        .map_err(|err| format!("failed to parse {}: {}", path.display(), err))?;
    Ok(config)
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
        let category = categorizer(rules, &description);
        let amount = rust_decimal::Decimal::from_str(&amount)?;

        transactions.push(Transaction {
            id: 0,
            date,
            amount,
            category,
            description,
            bank: "COMMONWEALTH".to_string(),
        });
    }

    sort_by_category(&mut transactions);

    Ok(transactions)
}

fn sort_by_category(transactions: &mut Vec<Transaction>) {
    transactions.sort_by(|a, b| a.category.cmp(&b.category));
}

fn categorizer(rules: &Vec<CategoryRule>, description: &String) -> String {
    let mut result = "Unknown".to_string();
    let mut last_prio_level = 0;

    for rule in rules {
        for keyword in &rule.keywords {
            if description
                .to_lowercase()
                .contains(keyword.to_lowercase().as_str())
                && rule.priority > last_prio_level
            {
                result = rule.subcategory.clone();
                last_prio_level = rule.priority;
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_rules() -> Vec<CategoryRule> {
        vec![
            CategoryRule {
                subcategory: "Groceries".to_string(),
                keywords: vec![
                    "woolworths".to_string(),
                    "coles".to_string(),
                    "pearl".to_string(),
                ],
                priority: 60,
            },
            CategoryRule {
                subcategory: "Fuel".to_string(),
                keywords: vec!["ampol".to_string(), "pearl".to_string()],
                priority: 70,
            },
        ]
    }

    #[test]
    fn assigns_category_on_keyword_match() {
        let rules = mock_rules();
        let description =
            "WOOLWORTHS 2764 NERANG QL AUS Card xx8935 Value Date: 31/07/2026".to_string();
        let result = categorizer(&rules, &description);
        assert_eq!(result, "Groceries")
    }

    #[test]
    fn assigns_higher_priority_on_multiple_keyword_match() {
        let rules = mock_rules();
        let description =
            "PEARL SOUTHPORT SOUTHPORT QL AUS Card xx5231 Value Date: 29/07/2026".to_string();
        let result = categorizer(&rules, &description);
        assert_eq!(result, "Fuel")
    }

    #[test]
    fn returns_unknown_when_no_match() {
        let rules = mock_rules();
        let description = "EasyPark".to_string();
        let result = categorizer(&rules, &description);
        assert_eq!(result, "Unknown")
    }
}
