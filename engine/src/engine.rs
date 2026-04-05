use crate::types::Account;
use std::collections::HashMap;

pub enum Asset {
    Base,
    Quote,
}

pub struct Engine {
    pub accounts: HashMap<u64, Account>,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, trader_id: u64, amount: u64, asset: Asset) {
        if amount == 0 {
            eprintln!("Invalid deposit amount");
            return;
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                account.base_qty_available += amount;
            }
            Asset::Quote => {
                account.quote_qty_available += amount;
            }
        }

        println!("Deposit successfull of {} to {}", amount, trader_id)
    }

    pub fn lock_funds(&mut self, trader_id: u64, amount: u64, asset: Asset) -> Result<(), String> {
        if amount == 0 {
            eprintln!("Invalid withdraw amount");
            return Err("Invalid amount".to_string());
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                if account.base_qty_available >= amount {
                    account.base_qty_locked += amount;
                    account.base_qty_available -= amount;
                    println!("Lock successfull of base qty {} from {}", amount, trader_id);

                    Ok(())
                } else {
                    Err("Not enough base funds".to_string())
                }
            }
            Asset::Quote => {
                if account.quote_qty_available >= amount {
                    account.quote_qty_locked += amount;
                    account.quote_qty_available -= amount;
                    println!(
                        "Lock successfull of quote qty {} from {}",
                        amount, trader_id
                    );

                    Ok(())
                } else {
                    Err("Not enough quote funds".to_string())
                }
            }
        }
    }

    pub fn withdraw(&mut self, trader_id: u64, amount: u64, asset: Asset) -> Result<(), String> {
        if amount == 0 {
            eprintln!("Invalid withdraw amount");
            return Err("Invalid amount".to_string());
        }

        let account = self.accounts.entry(trader_id).or_default();

        match asset {
            Asset::Base => {
                if account.base_qty_available >= amount {
                    account.base_qty_available -= amount;
                    println!("Withdraw successfull of base {} from {}", amount, trader_id);

                    Ok(())
                } else {
                    Err("Not enough base funds".to_string())
                }
            }
            Asset::Quote => {
                if account.quote_qty_available >= amount {
                    account.quote_qty_available -= amount;
                    println!(
                        "Withdraw successfull of quote {} from {}",
                        amount, trader_id
                    );

                    Ok(())
                } else {
                    Err("Not enough quote funds".to_string())
                }
            }
        }
    }
}
