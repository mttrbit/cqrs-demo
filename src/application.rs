#[cfg(test)]
mod simple_application_test {

    use crate::{
        aggregate::BankAccount, commands::OpenAccount, events::BankAccountEvent,
        queries::SimpleLoggingQueryProcessor,
    };
    use cqrs_es::{mem_store::MemStore, CqrsFramework};

    #[test]
    fn test_event_store_single_command() {
        let event_store = MemStore::<BankAccount, BankAccountEvent>::default();
        let query = SimpleLoggingQueryProcessor {};
        let cqrs = CqrsFramework::new(event_store, vec![Box::new(query)]);
        cqrs.execute(
            "test_id",
            OpenAccount {
                account_id: "100-123456789-001".to_string(),
            },
        )
        .unwrap()
    }
}
