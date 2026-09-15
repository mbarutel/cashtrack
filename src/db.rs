use std::{path::PathBuf, str::FromStr};

use crate::{
    models::{Transaction, TransactionDbRow},
    MyResult,
};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Row};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> MyResult<Database> {
        Database::prepare_path(&path)?;
        let conn = Connection::open(path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transactions (
          id           INTEGER PRIMARY KEY AUTOINCREMENT,
          date         TEXT NOT NULL,
          amount       TEXT NOT NULL,
          category     TEXT NOT NULL,
          description  TEXT NOT NULL,
          bank         TEXT NOT NULL,
          UNIQUE(date, amount, description, bank)
        );",
        )?;

        Ok(Database { conn })
    }

    fn prepare_path(path: &PathBuf) -> MyResult<()> {
        let parent = path
            .parent()
            .expect("Database cannot be located at root level");

        if !parent.is_dir() {
            std::fs::create_dir_all(parent)
                .map_err(|err| format!("{}: {}", parent.display(), err))?
        }

        Ok(())
    }

    pub fn insert_transactions(&mut self, transactions: &[Transaction]) -> MyResult<usize> {
        let tx = self.conn.transaction()?;
        let mut inserted = 0;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO transactions
                (date, amount, category, description, bank)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for t in transactions {
                inserted += stmt.execute(params![
                    t.date,
                    t.amount.to_string(),
                    t.category,
                    t.description,
                    t.bank
                ])?;
            }
        }

        tx.commit()?;

        Ok(inserted)
    }

    pub fn list_transactions(&mut self, from: &str, to: &str) -> MyResult<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                id,
                date,
                amount,
                category,
                description,
                bank
            FROM
                transactions
            WHERE
                date >= ?1 AND date <= ?2
            ORDER BY
                date
            DESC",
        )?;

        let rows = stmt.query_map(params![from, to], |row| TransactionDbRow::try_from(row))?;

        let mut transactions = Vec::new();

        for row in rows {
            transactions.push(Transaction::try_from(row?)?);
        }

        Ok(transactions)
    }

    pub fn list_all_transactions(&mut self) -> MyResult<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT
                id,
                date,
                amount,
                category,
                description,
                bank
            FROM
                transactions
            ORDER BY
                 date
            DESC",
        )?;

        let rows = stmt.query_map([], |row| TransactionDbRow::try_from(row))?;

        let mut transactions = Vec::new();

        for row in rows {
            transactions.push(Transaction::try_from(row?)?);
        }

        Ok(transactions)
    }

    pub fn count(&mut self) -> MyResult<usize> {
        unimplemented!()
    }

    pub fn last_row(&mut self) -> MyResult<Transaction> {
        unimplemented!()
    }
}

impl TryFrom<&Row<'_>> for TransactionDbRow {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(0)?,
            date: value.get(1)?,
            amount: value.get(2)?,
            category: value.get(3)?,
            description: value.get(4)?,
            bank: value.get(5)?,
        })
    }
}

pub struct TransactionDbRow {
    pub id: i64,
    pub date: String,
    pub amount: String,
    pub category: String,
    pub description: String,
    pub bank: String,
}

impl TryFrom<TransactionDbRow> for Transaction {
    type Error = String;

    fn try_from(value: TransactionDbRow) -> Result<Self, Self::Error> {
        let amount = rust_decimal::Decimal::from_str(&value.amount).map_err(|e| format!("{e}"))?;
        let date = NaiveDate::from_str(&value.date).map_err(|e| format!("{e}"))?;

        Ok(Self {
            date,
            amount: amount,
            category: value.category,
            description: value.description,
            bank: value.bank,
        })
    }
}
