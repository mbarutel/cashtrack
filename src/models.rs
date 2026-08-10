use chrono::NaiveDate;
use serde::Deserialize;

pub struct Transaction {
    pub id: i64,
    pub date: NaiveDate,
    pub amount: f64,
    pub category: String,
    pub description: String,
    pub bank: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTransaction {
    pub date: String,
    pub amount: f64,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: Vec<CategoryRule>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryRule {
    pub subcategory: String,
    pub keywords: Vec<String>,
    pub priority: u32,
}

// impl TryFrom<CreateTransaction> for Transaction {
//     fn from(value: CreateTransaction) -> Self {
//         Self {
//             id: 0,
//             date: todo!(),
//             amount: todo!(),
//             category: todo!(),
//             description: todo!(),
//             bank: todo!(),
//         }
//     }
// }
impl TryFrom<CreateTransaction> for Transaction {
    type Error = Box<dyn std::error::Error>;

    fn try_from(value: CreateTransaction) -> Result<Self, Self::Error> {
        let date = chrono::NaiveDate::parse_from_str(&value.date, "%d/%m/%Y")?;

        Ok(Self {
            id: 0,
            date,
            amount: value.amount,
            category: "Placeholder".to_string(),
            description: value.description,
            bank: "Placeholder".to_string(),
        })
    }
}
