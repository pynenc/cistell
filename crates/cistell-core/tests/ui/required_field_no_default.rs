use cistell_core::Config;

#[derive(Config)]
#[config(prefix = "APP", group = "redis")]
struct RequiredFieldNoDefault {
    host: String,
}

fn main() {}
