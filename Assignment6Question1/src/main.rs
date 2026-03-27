use rusqlite::{params, Connection};
use chrono::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UBaseErr {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("User not found")]
    UserNotFound,
}

pub struct Bank {
    conn: Connection,
}

impl Bank {
    pub fn new(conn: Connection) -> Result<Self, UBaseErr> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                balance INTEGER NOT NULL DEFAULT 1000
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transactions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_user TEXT NOT NULL,
                to_user TEXT NOT NULL,
                amount INTEGER NOT NULL,
                date TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Bank { conn })
    }

    pub fn add_user(&self, username: &str, password: &str) -> Result<(), UBaseErr> {
        self.conn.execute(
            "INSERT INTO users (username, password, balance) VALUES (?1, ?2, 1000)",
            params![username, password],
        )?;
        Ok(())
    }

    pub fn get_balance(&self, username: &str) -> Result<i32, UBaseErr> {
        let mut stmt = self.conn.prepare("SELECT balance FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Err(UBaseErr::UserNotFound)
        }
    }

    pub fn pay(&self, from_user: &str, to_user: &str, amount: i32) -> Result<(), UBaseErr> {
        self.conn.execute("BEGIN TRANSACTION", [])?;

        let from_balance = self.get_balance(from_user);
        match from_balance {
            Ok(b) if b >= amount => {
                let _ = self.conn.execute(
                    "UPDATE users SET balance = balance - ?1 WHERE username = ?2",
                    params![amount, from_user],
                );
                let _ = self.conn.execute(
                    "UPDATE users SET balance = balance + ?1 WHERE username = ?2",
                    params![amount, to_user],
                );
                
                // Format: 12/10/2019 11:34 p.m.
                let now = Local::now().format("%d/%m/%Y %I:%M %p").to_string().to_lowercase();
                
                let _ = self.conn.execute(
                    "INSERT INTO transactions (from_user, to_user, amount, date) VALUES (?1, ?2, ?3, ?4)",
                    params![from_user, to_user, amount, now],
                );
                
                self.conn.execute("COMMIT", [])?;
                Ok(())
            }
            Ok(_) => {
                self.conn.execute("ROLLBACK", [])?;
                Err(UBaseErr::InsufficientFunds)
            }
            Err(e) => {
                self.conn.execute("ROLLBACK", [])?;
                Err(e)
            }
        }
    }

    pub fn get_transactions_history(&self, u_name: &str) -> Result<(), UBaseErr> {
        let mut stmt = self.conn.prepare(
            "SELECT from_user, to_user, amount, date FROM transactions WHERE from_user = ?1 OR to_user = ?1 ORDER BY id ASC"
        )?;

        let tx_iter = stmt.query_map(params![u_name], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;

        for tx in tx_iter {
            let (from_u, to_u, amt, date) = tx?;
            if to_u == u_name {
                println!("{} received ${} from {} on {}", u_name, amt, from_u, date);
            } else if from_u == u_name {
                println!("{} sent ${} to {} on {}", u_name, amt, to_u, date);
            }
        }

        Ok(())
    }
}

fn main() {
    std::fs::create_dir_all("data").unwrap_or_default();
    let conn = Connection::open("data/bank.db").expect("Failed to open DB");
    let _bank = Bank::new(conn).expect("Failed to initialize DB");
    println!("Assignment 6 Question 1 initialized successfully.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> Bank {
        let conn = Connection::open_in_memory().unwrap();
        Bank::new(conn).unwrap()
    }

    #[test]
    fn test_pay_success_and_history() {
        let bank = setup_db();
        bank.add_user("Matt", "123").unwrap();
        bank.add_user("Jim", "123").unwrap();
        
        assert_eq!(bank.get_balance("Matt").unwrap(), 1000);
        
        bank.pay("Jim", "Matt", 140).unwrap();
        
        assert_eq!(bank.get_balance("Matt").unwrap(), 1140);
        assert_eq!(bank.get_balance("Jim").unwrap(), 860);
        
        assert!(bank.get_transactions_history("Matt").is_ok());
    }

    #[test]
    fn test_pay_insufficient_funds() {
        let bank = setup_db();
        bank.add_user("Matt", "123").unwrap();
        bank.add_user("Jim", "123").unwrap();
        
        let res = bank.pay("Matt", "Jim", 2000);
        assert!(matches!(res, Err(UBaseErr::InsufficientFunds)));
        
        assert_eq!(bank.get_balance("Matt").unwrap(), 1000);
        assert_eq!(bank.get_balance("Jim").unwrap(), 1000);
    }
}
