use cistell_core::Config;

#[derive(Config)]
#[config(prefix = "APP", group = "redis")]
struct SkipWithDefault {
    #[config(skip, default = "x")]
    host: String,
}

fn main() {}
