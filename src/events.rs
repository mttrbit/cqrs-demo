use crate::aggregate::BankAccount;
use cqrs_es::DomainEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccountOpened {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomerDepositedMoney {
    pub amount: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomerWithdrewMoney {
    pub amount: f64,
    pub balance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BankAccountEvent {
    AccountOpened(AccountOpened),
    CustomerDepositedMoney(CustomerDepositedMoney),
    CustomerWithdrewMoney(CustomerWithdrewMoney),
}

impl DomainEvent<BankAccount> for BankAccountEvent {
    fn apply(self, account: &mut BankAccount) {
        match self {
            BankAccountEvent::AccountOpened(e) => e.apply(account),
            BankAccountEvent::CustomerDepositedMoney(e) => e.apply(account),
            BankAccountEvent::CustomerWithdrewMoney(e) => e.apply(account),
        }
    }
}

impl DomainEvent<BankAccount> for AccountOpened {
    fn apply(self, _account: &mut BankAccount) {}
}

impl DomainEvent<BankAccount> for CustomerDepositedMoney {
    fn apply(self, account: &mut BankAccount) {
        account.balance = self.balance;
    }
}

impl DomainEvent<BankAccount> for CustomerWithdrewMoney {
    fn apply(self, account: &mut BankAccount) {
        account.balance = self.balance;
    }
}
