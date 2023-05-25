use conf_macros::Config;

#[derive(Config)]
union TestConfig {
    foo: u32,
    bar: u32,
}

fn main() {}
