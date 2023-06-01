use conf_macros::Config;

#[derive(Config)]
pub struct TestConfig {
    pub field1: u32,
    #[check(|foo: u32, bar:u32| todo!())]
    pub field2: u32,
    pub field3: u32,
}

fn main() {}
