use rusqlite::{params, Connection};
use std::env;
use rpassword::prompt_password;
use bcrypt::{hash, verify, DEFAULT_COST};
use thiserror::Error;
use chrono::prelude::*;

#[derive(Error, Debug)]
pub enum UBaseErr {
    #[error("Database error: {0}")]
    DbError(#[from] rusqlite::Error),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("User not found")]
    UserNotFound,
    #[error("Bcrypt error: {0}")]
    BcryptError(#[from] bcrypt::BcryptError),
    #[error("Invalid password")]
    InvalidPassword,
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
        let hashed = hash(password, DEFAULT_COST)?;
        self.conn.execute(
            "INSERT INTO users (username, password, balance) VALUES (?1, ?2, 1000)",
            params![username, hashed],
        )?;
        Ok(())
    }
    
    pub fn get_user_password_hash(&self, username: &str) -> Result<String, UBaseErr> {
        let mut stmt = self.conn.prepare("SELECT password FROM users WHERE username = ?1")?;
        let mut rows = stmt.query(params![username])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            Err(UBaseErr::UserNotFound)
        }
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
}

fn main() {
    std::fs::create_dir_all("data").unwrap_or_default();
    let conn = Connection::open("data/bank.db").expect("Failed to open DB");
    let bank = Bank::new(conn).expect("Failed to initialize DB");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please provide a command: new, transfer, or balance");
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "new" => {
            if args.len() < 4 {
                println!("Please complete the command. Usage: new <username> <password>");
                return;
            }
            let username = &args[2];
            let password = &args[3];
            println!("Adding user {} with password {}…", username, password);
            match bank.add_user(username, password) {
                Ok(_) => println!("Operation done successfully!"),
                Err(e) => println!("Error adding user: {:?}", e),
            }
        }
        "transfer" => {
            if args.len() < 5 {
                println!("Please complete the command. Usage: transfer <from_user> <to_user> <amount>");
                return;
            }
            let from_user = &args[2];
            let to_user = &args[3];
            let amount: i32 = match args[4].parse() {
                Ok(v) => v,
                Err(_) => {
                    println!("Amount must be a number");
                    return;
                }
            };

            let pw_prompt = prompt_password("Please input your password: ").unwrap_or_default();
            let hash_res = bank.get_user_password_hash(from_user);
            match hash_res {
                Ok(h) => {
                    if verify(&pw_prompt, &h).unwrap_or(false) {
                        println!("Sending money from {} to {}…", from_user, to_user);
                        match bank.pay(from_user, to_user, amount) {
                            Ok(_) => println!("Operation done successfully!"),
                            Err(e) => println!("Error transferring: {:?}", e),
                        }
                    } else {
                        println!("Invalid password");
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        "balance" => {
            if args.len() < 3 {
                println!("Please complete the command. Usage: balance <username>");
                return;
            }
            let username = &args[2];
            let pw_prompt = prompt_password("Please input your password: ").unwrap_or_default();
            let hash_res = bank.get_user_password_hash(username);
            match hash_res {
                Ok(h) => {
                    if verify(&pw_prompt, &h).unwrap_or(false) {
                        match bank.get_balance(username) {
                            Ok(bal) => {
                                println!("Balance is ${}", bal);
                                println!("Operation done successfully!");
                            }
                            Err(e) => println!("Error getting balance: {:?}", e),
                        }
                    } else {
                        println!("Invalid password");
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            }
        }
        _ => {
            println!("Unknown command");
        }
    }
}
