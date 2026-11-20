use crate::Output;

fn sample() -> Output {
    Output {
        status: 0,
        stdout: b"out".to_vec(),
        stderr: b"err".to_vec(),
    }
}

#[test]
fn equal_when_every_field_matches() {
    assert_eq!(
        sample(),
        Output {
            status: 0,
            stdout: b"out".to_vec(),
            stderr: b"err".to_vec()
        }
    );
}

#[test]
fn differs_on_any_field() {
    assert_ne!(sample(), Output { status: 1, ..sample() });
    assert_ne!(sample(), Output { stdout: b"other".to_vec(), ..sample() });
    assert_ne!(sample(), Output { stderr: Vec::new(), ..sample() });
}
