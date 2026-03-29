use cistell_core::Config;

#[derive(Config)]
#[config(group = "redis")]
struct MissingPrefix {
    #[config(default = "localhost")]
    host: String,
}

fn main() {}
