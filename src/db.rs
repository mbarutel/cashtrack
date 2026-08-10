use std::{path::Path, str::FromStr};

use rusqlite::{Connection, Row, params};
use rust_decimal::Decimal;

use crate::{MyResult, models::Transaction};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> MyResult<Database> {
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

    // pub fn list_transactions(
    //     conn: &mut Connection,
    //     from: NaiveDate,
    //     to: NaiveDate,
    // ) -> MyResult<Vec<Transaction>> {
    //     let mut stmt = conn.prepare(
    //         "SELECT id, date, amount, category, description, bank
    //     FROM transactions
    //     WHERE date >= ?1 AND date <= ?2
    //     ORDER BY date, id",
    //     )?;

    //     let rows = stmt.query_map(params![from, to], |row| Self::parse_transaction(row))?;

    //     rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    // }

    pub fn list_all_transactions(&mut self) -> MyResult<Vec<Transaction>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, date, amount, category, description, bank
        FROM transactions
        ORDER BY date, id",
        )?;

        let rows = stmt.query_map(params![], |row| Self::parse_transaction(row))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    fn parse_transaction(row: &Row) -> rusqlite::Result<Transaction> {
        let amount: String = row.get(2)?;
        let amount = Decimal::from_str(&amount).map_err(|err| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(err))
        })?;

        Ok(Transaction {
            id: row.get(0)?,
            date: row.get(1)?,
            amount,
            category: row.get(3)?,
            description: row.get(4)?,
            bank: row.get(5)?,
        })
    }
}
