# ECE 421 - Assignment 6: Databases

This repository contains the implementation for Assignment 6, focusing on database integration with SQLite and secure password hashing.

## Project Structure

The assignment is divided into three separate Cargo projects as requested:

- **[Assignment6Question1](./Assignment6Question1)**: Core functionality.
  - Implements a SQLite-backed banking system using `rusqlite`.
  - Includes a `pay` function with balance validation.
  - Includes `get_transactions_history` to display formatted history.
  - Contains unit tests for all core logic.
- **[Assignment6Question2](./Assignment6Question2)**: Command-line interface.
  - Adds a CLI wrapper for the database.
  - Implements `new`, `transfer`, and `balance` commands.
  - Uses `bcrypt` for password hashing and `rpassword` for secure CLI input.
- **[Assignment6Question3](./Assignment6Question3)**: Security Upgrade.
  - Replaces `bcrypt` with `argon2` (best practice) for password hashing and verification.
  - Includes secure salt generation.

## How to Run & Test

### Question 1 (Unit Tests)
Validates the database logic and history formatting.
```bash
cd Assignment6Question1
cargo test -- --nocapture
```

### Question 2 & 3 (CLI Application)
Interact with the database via the terminal. Both projects support the same commands.
```bash
cd Assignment6Question3 # Or Question2
# Create a new user
cargo run new matt mattpw

# Check balance (prompts for password)
cargo run balance matt

# Send money
cargo run new jim jimpw
cargo run transfer matt jim 100
```

## Dependencies
- `rusqlite` (with `bundled` feature)
- `bcrypt` / `argon2`
- `rpassword`
- `thiserror`
- `chrono`
- `rand`