use crate::kept_bytes::kept_bytes;
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn returns_the_contents() {
    let kept = Mutex::new(vec![1u8, 2, 3]);
    assert_eq!(kept_bytes(&kept), vec![1u8, 2, 3]);
}

#[test]
fn the_copy_does_not_follow_later_changes() {
    let kept = Mutex::new(vec![1u8, 2, 3]);
    let copy = kept_bytes(&kept);
    kept.lock().unwrap().push(4);
    assert_eq!(copy, vec![1u8, 2, 3]);
}

#[test]
fn a_poisoned_lock_still_gives_its_bytes() {
    let kept = Arc::new(Mutex::new(vec![5u8, 6]));
    let holder = Arc::clone(&kept);
    let joined = thread::spawn(move || {
        let _guard = holder.lock().unwrap();
        panic!("poison the lock");
    })
    .join();
    assert!(joined.is_err());
    assert!(kept.is_poisoned());
    assert_eq!(kept_bytes(&kept), vec![5u8, 6]);
}
