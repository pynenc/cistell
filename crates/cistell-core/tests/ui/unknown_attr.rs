use cistell_core::Config;

#[derive(Config)]
#[config(prefix = "APP", group = "redis", bogus = "x")]
struct UnknownAttr {
    #[config(default = "localhost")]
    host: String,
}

fn main() {}
