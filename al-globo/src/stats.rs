use std::collections::HashMap;
use actix::{Actor, Handler, SyncContext, Message};
use chrono::{DateTime, Local};

#[derive(Debug)]
pub struct TransactionStatistics {
    start_time: Option<DateTime<Local>>,
    end_time: Option<DateTime<Local>>,
}

#[derive(Debug)]
pub struct Stats {
    pub repo: HashMap::<String, TransactionStatistics>,
}

impl Actor for Stats {
    type Context = SyncContext<Self>;
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            repo: HashMap::new()
        }
    }

    pub fn set_start_time(&mut self, transaction_id: String) {
        let new_stat = TransactionStatistics {
            start_time: Some(Local::now()),
            end_time: None,
        };

        self.repo.insert(transaction_id, new_stat);
    }

    pub(crate) fn set_end_time(&mut self, transaction_id: &str) {
        if let Some(stat) = self.repo.get_mut(transaction_id) {
            stat.end_time = Some(Local::now());
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartTime(pub String);

impl Handler<StartTime> for Stats {
    type Result = ();

    fn handle(&mut self, msg: StartTime, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction_id = msg.0;

        self.set_start_time(transaction_id)
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StopTime(pub String);

impl Handler<StopTime> for Stats {
    type Result = ();
    fn handle(&mut self, msg: StopTime, _ctx: &mut SyncContext<Self>) -> Self::Result {
        let transaction_id = msg.0;

        self.set_end_time(&transaction_id);
        // TODO: implementar trait display
        println!("{:?}", self.repo);
    }
}