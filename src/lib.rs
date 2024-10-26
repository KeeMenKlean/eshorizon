mod uuid;
mod codec_main;
mod aggregatestore;
mod aggregate;
mod entity;
mod event;
mod command_main;
mod command_check;
mod commandhandler;
mod compare;
mod context;
mod eventbus;
mod eventhandler;
mod eventmaintenance;
mod eventsource;
mod eventstore;
mod matcher;
mod middleware;
mod outbox;
mod repo;
mod snapshot;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}