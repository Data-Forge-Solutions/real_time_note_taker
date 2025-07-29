#![warn(clippy::pedantic)]
/// Simple structure used for demonstration purposes.
#[derive(Debug, Clone, Default)]
struct MyStruct {
    value: u64,
    count: u32,
}

impl MyStruct {
    /// Creates a new `MyStruct` with the provided values.
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
