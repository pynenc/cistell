use cistell_core::Config;

#[derive(Config)]
#[config(prefix = "APP")]
struct MissingGroup {
    #[config(default = "localhost")]
    host: String,
}

fn main() {}
