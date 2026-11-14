//! decode_formatter: null is None, else exactly a non-empty command and a timeout.

use super::{decode_formatter, ConfigJsonValue as V, FormatterConfig};

fn formatter(command: &[&str], timeout: V) -> V {
    V::Object(vec![
        ("timeout_seconds".to_string(), timeout),
        ("command".to_string(), V::Array(command.iter().map(|c| V::String(c.to_string())).collect())),
    ])
}

#[test]
fn null_is_no_formatter() {
    assert_eq!(decode_formatter(Some(V::Null)), Ok(None));
}

#[test]
fn an_object_is_the_formatter() {
    assert_eq!(
        decode_formatter(Some(formatter(&["cargo", "fmt"], V::Number(30)))),
        Ok(Some(FormatterConfig { command: vec!["cargo".to_string(), "fmt".to_string()], timeout_seconds: 30 }))
    );
}

#[test]
fn refusals_name_the_key() {
    assert_eq!(decode_formatter(None), Err("missing key \"formatter\"".to_string()));
    assert_eq!(decode_formatter(Some(V::Number(1))), Err("formatter is neither null nor an object".to_string()));
    assert_eq!(
        decode_formatter(Some(formatter(&[], V::Number(1)))),
        Err("formatter.command is empty; it must name the formatter's program".to_string())
    );
    assert_eq!(
        decode_formatter(Some(formatter(&["f"], V::Null))),
        Err("formatter.timeout_seconds is not a number".to_string())
    );
}
