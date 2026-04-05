mod engine;
mod types;

use crate::engine::{Engine, Asset};

fn main() {
    let mut engine = Engine::new();
    
    // 1. Deposit
    engine.deposit(111, 100, Asset::Base);
    engine.deposit(111, 100, Asset::Quote);

    // 2. Withdraw (Using .unwrap() to handle the Result)
    engine.withdraw(111, 20, Asset::Base).unwrap();
    engine.withdraw(111, 20, Asset::Quote).unwrap();

    // 3. Lock Funds
    engine.lock_funds(111, 50, Asset::Base).unwrap();
    engine.lock_funds(111, 50, Asset::Quote).unwrap();
    
    println!("Account State: {:#?}", engine.accounts.get(&111).unwrap());
}