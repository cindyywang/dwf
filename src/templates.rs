use crate::config::Config;

pub fn list_templates() -> &'static [&'static str] {
    &["rust-default", "rust-ci-strict", "rust-fast-iter"]
}

pub fn get_template(name: &str) -> Option<Config> {
    match name {
        "rust-default" => Some(Config::default()),
        "rust-ci-strict" => {
            let mut c = Config::default();
            c.pipeline.all_features_in_full = true;
            c.pipeline.clippy_deny_warnings = true;
            Some(c)
        }
        "rust-fast-iter" => {
            let mut c = Config::default();
            c.pipeline.all_features_in_full = false;
            c.pipeline.clippy_deny_warnings = false; // faster/less strict locally
            Some(c)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_default_exists() {
        assert!(get_template("rust-default").is_some());
    }

    #[test]
    fn unknown_template_none() {
        assert!(get_template("nope").is_none());
    }

    #[test]
    fn list_contains_default() {
        let list = list_templates();
        assert!(list.contains(&"rust-default"));
    }
}
