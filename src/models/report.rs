use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::NaiveDate;
use rust_decimal::Decimal;

use super::{Transaction, UNCATEGORISED};

pub const LARGEST_N: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategorySummary {
    pub name: String,
    pub total: Decimal,
    pub count: usize,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub income: Decimal,
    pub spending: Decimal,
    pub transfers_in: Decimal,
    pub transfers_out: Decimal,
    pub spending_by_category: Vec<CategorySummary>,
    pub income_by_category: Vec<CategorySummary>,
    pub largest_debits: Vec<Transaction>,
    pub daily_spending: BTreeMap<NaiveDate, Decimal>,
}

impl Report {
    pub fn new(transactions: &[Transaction], transfer_categories: &HashSet<String>) -> Self {
        let mut report = Report::default();
        let mut spending: HashMap<&str, CategorySummary> = HashMap::new();
        let mut income: HashMap<&str, CategorySummary> = HashMap::new();
        let mut debits: Vec<&Transaction> = Vec::new();

        for t in transactions {
            report.start_date = Some(report.start_date.map_or(t.date, |d| d.min(t.date)));
            report.end_date = Some(report.end_date.map_or(t.date, |d| d.max(t.date)));

            if t.amount.is_zero() {
                continue;
            }

            let is_transfer = transfer_categories.contains(&t.category);
            let is_debit = t.amount < Decimal::ZERO;
            let magnitude = t.amount.abs();

            match (is_transfer, is_debit) {
                (true, true) => report.transfers_out += magnitude,
                (true, false) => report.transfers_in += magnitude,
                (false, true) => {
                    report.spending += magnitude;
                    *report.daily_spending.entry(t.date).or_default() += magnitude;
                    debits.push(t);
                    bump(&mut spending, &t.category, magnitude);
                }
                (false, false) => {
                    report.income += magnitude;
                    bump(&mut income, &t.category, magnitude);
                }
            }
        }

        report.spending_by_category = sorted(spending);
        report.income_by_category = sorted(income);

        debits.sort_by(|a, b| a.amount.cmp(&b.amount));
        report.largest_debits = debits.into_iter().take(LARGEST_N).cloned().collect();

        report
    }

    pub fn net(&self) -> Decimal {
        self.income - self.spending
    }

    pub fn savings_rate(&self) -> Option<Decimal> {
        // (!self.income.is_zero()).then(|| self.net() / self.income * Decimal::ONE_HUNDRED)

        match self.income.is_zero() {
            true => None,
            false => Some(self.net() / self.income * Decimal::ONE_HUNDRED),
        }
    }

    pub fn uncategorised(&self) -> Option<&CategorySummary> {
        self.spending_by_category
            .iter()
            .find(|c| c.name == UNCATEGORISED)
    }

    pub fn share(&self, category: &CategorySummary) -> Decimal {
        match self.spending.is_zero() {
            true => Decimal::ZERO,
            false => category.total / self.spending * Decimal::ONE_HUNDRED,
        }
    }
}

fn bump<'a>(map: &mut HashMap<&'a str, CategorySummary>, category: &'a str, magnitude: Decimal) {
    let entry = map.entry(category).or_insert_with(|| CategorySummary {
        name: category.to_string(),
        total: Decimal::ZERO,
        count: 0,
    });

    entry.total += magnitude;
    entry.count += 1;
}

fn sorted(map: HashMap<&str, CategorySummary>) -> Vec<CategorySummary> {
    // let mut summaries = map
    //     .into_iter()
    //     .map(|(_, val)| val)
    //     .collect::<Vec<CategorySummary>>();
    // summaries.sort_unstable_by(|a, b| a.total.cmp(&b.total));

    let mut values: Vec<CategorySummary> = map.into_values().collect();
    values.sort_by(|a, b| a.total.cmp(&b.total).then_with(|| a.name.cmp(&b.name)));

    values
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tx(id: i64, day: u32, amount: &str, category: &str) -> Transaction {
        Transaction {
            id,
            date: NaiveDate::from_ymd_opt(2026, 8, day).unwrap(),
            amount: amount.parse().unwrap(),
            category: category.into(),
            description: format!("tx {id}"),
            bank: "Commonwealth".into(),
        }
    }

    fn dec(s: &str) -> Decimal {
        s.parse().unwrap()
    }

    fn transfers() -> HashSet<String> {
        ["Money Transfer".to_string()].into_iter().collect()
    }

    #[test]
    fn empty_report_has_no_dates_and_zero_totals() {
        let report = Report::new(&[], &transfers());

        assert_eq!(report.start_date, None);
        assert_eq!(report.spending, Decimal::ZERO);
        assert_eq!(report.savings_rate(), None);
    }

    #[test]
    fn first_transaction_is_counted() {
        let report = Report::new(&[tx(1, 3, "-10", "Groceries")], &transfers());
        assert_eq!(report.spending, dec("-10"));
        assert_eq!(report.spending_by_category[0].count, 1);
        assert_eq!(
            report.start_date,
            Some(NaiveDate::from_ymd_opt(2026, 8, 3).unwrap())
        )
    }

    #[test]
    fn transfers_are_excluded_from_income_and_spending() {
        let report = Report::new(
            &[
                tx(1, 1, "3000", "Mike In"),
                tx(2, 2, "-500", "Money Transfer"),
                tx(3, 3, "-200", "Rent"),
            ],
            &transfers(),
        );
        assert_eq!(report.income, dec("3000"));
        assert_eq!(report.spending, dec("200"));
        assert_eq!(report.transfers_out, dec("500"));
        assert_eq!(report.net(), dec("2800"));
        assert!(report
            .spending_by_category
            .iter()
            .all(|c| c.name != "Money Transfer"));
    }

    #[test]
    fn categories_sorted_by_total_with_counts_and_share() {
        let report = Report::new(
            &[
                tx(1, 1, "-10", "Eating Out"),
                tx(2, 2, "-15", "Eating Out"),
                tx(3, 3, "-75", "Rent"),
            ],
            &transfers(),
        );
        let names = report
            .spending_by_category
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<&str>>();
        assert_eq!(names, ["Eating Out", "Rent"]);
        assert_eq!(report.spending_by_category[1].count, 2);
        assert_eq!(report.share(&report.spending_by_category[0]), dec("75"));
    }

    #[test]
    fn largest_debits_are_capped_and_ordered() {
        let txs: Vec<Transaction> = (1..=7)
            .map(|i| tx(i, i as u32, &format!("-{}", i * 10), "Misc"))
            .collect();
        let r = Report::new(&txs, &transfers());
        let amounts: Vec<Decimal> = r.largest_debits.iter().map(|t| t.amount).collect();
        assert_eq!(
            amounts,
            [dec("-70"), dec("-60"), dec("-50"), dec("-40"), dec("-30")]
        );
    }

    #[test]
    fn daily_spending_sums_same_day_debits_only() {
        let r = Report::new(
            &[
                tx(1, 4, "-5", "A"),
                tx(2, 4, "-7", "B"),
                tx(3, 4, "100", "Income"),
            ],
            &transfers(),
        );
        let day = NaiveDate::from_ymd_opt(2026, 8, 4).unwrap();
        assert_eq!(r.daily_spending[&day], dec("12"));
        assert_eq!(r.daily_spending.len(), 1);
    }

    #[test]
    fn savings_rate_is_net_over_income() {
        let r = Report::new(
            &[tx(1, 1, "1000", "Pay"), tx(2, 2, "-250", "Rent")],
            &transfers(),
        );
        assert_eq!(r.savings_rate(), Some(dec("75")));
        assert_eq!(r.uncategorised(), None);
    }
}
