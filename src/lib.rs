mod uuid;
mod codec_main;
mod aggregatestore;
mod aggregate;
mod entity;
mod event;
mod command_main;
mod command_check;
mod commandhandler;

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