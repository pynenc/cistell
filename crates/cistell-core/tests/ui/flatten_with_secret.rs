use cistell_core::Config;

#[derive(Config)]
#[config(prefix = "APP", group = "inner")]
struct Inner {
    #[config(default = false)]
    enabled: bool,
}

#[derive(Config)]
#[config(prefix = "APP", group = "outer")]
struct FlattenWithSecret {
    #[config(flatten, secret)]
    inner: Inner,
}

fn main() {}
