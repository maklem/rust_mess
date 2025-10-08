use std::cell::RefCell;
use mess_lib::reset_on_failure_count::ResettingCounter;

#[test]
fn test_reset_on_failure_count() {
    let is_reset = RefCell::new(false);
    
    let reset_function = || {
        *is_reset.borrow_mut() = true;
    };
    let mut counter = ResettingCounter::new(reset_function, 2);

    assert!(!*is_reset.borrow());

    counter.increment_failure();
    assert!(!*is_reset.borrow());

    counter.increment_failure();
    assert!(*is_reset.borrow());
}

#[test]
fn test_reset_on_failure_count_reset() {
    let is_reset = RefCell::new(false);
    let reset_function = || {
        *is_reset.borrow_mut() = true;
    };
    let mut counter = ResettingCounter::new( reset_function, 2);

    assert!(! *is_reset.borrow());

    counter.increment_failure();
    assert!(! *is_reset.borrow());

    counter.reset();

    counter.increment_failure();
    assert!(! *is_reset.borrow());
}