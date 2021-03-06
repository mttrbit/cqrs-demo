use crate::{aggregate::BankAccount, events::BankAccountEvent};
use cqrs_es::{EventEnvelope, Query, QueryProcessor};
use serde::{Deserialize, Serialize};

pub struct SimpleLoggingQueryProcessor {}

impl QueryProcessor<BankAccount, BankAccountEvent> for SimpleLoggingQueryProcessor {
    fn dispatch(
        &self,
        aggregate_id: &str,
        events: &[EventEnvelope<BankAccount, BankAccountEvent>],
    ) {
        for event in events {
            let payload = serde_json::to_string_pretty(&event.payload).unwrap();
            println!("{}-{}\n{}", aggregate_id, event.sequence, payload);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccountQuery {
    account_id: Option<String>,
    balance: f64,
    written_checks: Vec<String>,
}

impl Query<BankAccount, BankAccountEvent> for BankAccountQuery {
    fn update(&mut self, event: &EventEnvelope<BankAccount, BankAccountEvent>) {
        match &event.payload {
            BankAccountEvent::AccountOpened(payload) => {
                self.account_id = Some(payload.account_id.clone());
            }
            BankAccountEvent::CustomerDepositedMoney(payload) => {
                self.balance = payload.balance;
            }
            BankAccountEvent::CustomerWithdrewMoney(payload) => {
                self.balance = payload.balance;
            }
        }
    }
}

impl Default for BankAccountQuery {
    fn default() -> Self {
        BankAccountQuery {
            account_id: None,
            balance: 0_f64,
            written_checks: Default::default(),
        }
    }
}
