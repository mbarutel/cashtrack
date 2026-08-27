use crate::{
    MyResult,
    models::{Transaction, TransactionDbRow},
};
use chrono::NaiveDate;
use rusqlite::{Connection, Row, params};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> MyResult<Database> {
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

    pub fn list_transactions(
        conn: &mut Connection,
        from: NaiveDate,
        to: NaiveDate,
    ) -> MyResult<Vec<Transaction>> {
        let mut stmt = conn.prepare(
            "SELECT id, date, amount, category, description, bank
        FROM transactions
        WHERE date >= ?1 AND date <= ?2
        ORDER BY date, id",
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
                 date,
                 id",
        )?;

        let rows = stmt.query_map([], |row| TransactionDbRow::try_from(row))?;

        let mut transactions = Vec::new();

        for row in rows {
            transactions.push(Transaction::try_from(row?)?);
        }

        Ok(transactions)
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
