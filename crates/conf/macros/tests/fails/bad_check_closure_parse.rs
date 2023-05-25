use conf_macros::Config;

#[derive(Config)]
pub struct TestConfig {
    pub field1: u32,
    #[check(foobar)]
    pub field2: u32,
    pub field3: u32,
}

fn main() {}
