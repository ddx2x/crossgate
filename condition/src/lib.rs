pub mod cond;
pub use cond::*;

pub mod validate;
pub use validate::*;

fn remove_apostrophe(s: String) -> String {
    if s.starts_with(r#"'"#) {
        s.trim_end_matches(r#"'"#)
            .to_string()
            .trim_start_matches("'")
            .to_string()
    } else {
        s.trim_end_matches(r#"""#)
            .to_string()
            .trim_start_matches(r#"""#)
            .to_string()
    }
}
