use anyhow::{Context, Result};
use rust_decimal::Decimal;
use std::{path::PathBuf, str::FromStr};

use crate::models::{transaction::NewTransaction, Transaction};
use chrono::NaiveDate;
use rusqlite::{params, Connection, Row};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: PathBuf) -> Result<Database> {
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

    fn prepare_path(path: &PathBuf) -> Result<()> {
        let parent = path
            .parent()
            .expect("Database cannot be located at root level");

        if !parent.is_dir() {
            std::fs::create_dir_all(parent).with_context(|| format!("{}", parent.display()))?
        }

        Ok(())
    }

    pub fn insert_transactions(&mut self, transactions: &[NewTransaction]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut inserted = 0;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO transactions
                (date, amount, category, description, bank)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            )?;

            for t in transactions {
                println!("{:?}", t);
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

    pub fn list_transactions(
        &mut self,
        from: &NaiveDate,
        to: &NaiveDate,
    ) -> Result<Vec<Transaction>> {
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

        let mut rows = stmt.query(params![from, to])?;
        let mut transactions = Vec::new();

        while let Some(row) = rows.next()? {
            transactions.push(Transaction::try_from(row)?);
        }

        Ok(transactions)
    }

    pub fn list_all_transactions(&mut self) -> Result<Vec<Transaction>> {
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

        let mut rows = stmt.query([])?;
        let mut transactions = Vec::new();

        while let Some(row) = rows.next()? {
            transactions.push(Transaction::try_from(row)?);
        }

        Ok(transactions)
    }

    pub fn count(&mut self) -> Result<usize> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM transactions")?;

        let count: i64 = stmt.query_row([], |row| row.get(0))?;

        Ok(count as usize)
    }

    pub fn last_row(&mut self) -> Result<Transaction> {
        // Do we want the latest row inserted, latest row based on date?
        // And do we consider bank?
        unimplemented!()
    }
}

impl TryFrom<&Row<'_>> for Transaction {
    type Error = anyhow::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let date = value.get::<_, String>(1)?;
        let amount = value.get::<_, String>(2)?;

        Ok(Self {
            id: value.get(0)?,
            date: NaiveDate::from_str(&date)?,
            amount: Decimal::from_str(&amount)?,
            category: value.get(3)?,
            description: value.get(4)?,
            bank: value.get(5)?,
        })
    }
}
