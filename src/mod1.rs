#![warn(clippy::pedantic)]
#[derive(Debug, Clone, Default)]
struct MyStruct {
    value: u64,
    count: u32,
}

impl MyStruct {
    fn new(value: u64, count: u32) -> Self {
        Self { value, count }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_test() {

        assert_eq!(0, 0);
    }
}
