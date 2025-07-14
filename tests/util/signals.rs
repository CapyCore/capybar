#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use capybar::util::signals::Signal;

    #[test]
    fn basic_usage() {
        let signal = Signal::new();
        let last_value = Rc::new(RefCell::new(None));
        let last_value_clone = Rc::clone(&last_value);

        signal.connect(move |data| {
            if let Some(value) = data.downcast_ref::<i32>() {
                *last_value_clone.borrow_mut() = Some(*value);
            }
        });

        assert!(last_value.borrow().is_none());

        signal.emit(&42i32);
        assert_eq!(*last_value.borrow(), Some(42));

        signal.emit(&100i32);
        assert_eq!(*last_value.borrow(), Some(100));
    }

    #[test]
    fn reacts_to_correct_type() {
        let signal = Signal::new();
        let state = Rc::new(RefCell::new(0));
        let state_clone = Rc::clone(&state);

        signal.connect(move |data| {
            if let Some(value) = data.downcast_ref::<i32>() {
                *state_clone.borrow_mut() += value;
            }
        });

        signal.emit(&42i32);
        signal.emit(&"ignore");
        assert_eq!(*state.borrow(), 42);
    }

    #[test]
    fn ignores_wrong_type() {
        let signal = Signal::new();
        let called = Rc::new(RefCell::new(true));
        let called_clone = Rc::clone(&called);

        signal.connect(move |data| {
            if data.downcast_ref::<bool>().is_some() {
                *called_clone.borrow_mut() = false;
            }
        });

        signal.emit(&"not a bool");
        assert!(*called.borrow());
    }

    #[test]
    fn latest_emit_applied() {
        let signal = Signal::new();
        let value = Rc::new(RefCell::new(0));
        let value_clone = Rc::clone(&value);

        signal.connect(move |data| {
            if let Some(v) = data.downcast_ref::<i32>() {
                *value_clone.borrow_mut() = *v;
            }
        });

        signal.emit(&1i32);
        signal.emit(&2i32);
        signal.emit(&3i32);

        assert_eq!(*value.borrow(), 3);
    }

    #[test]
    fn no_panic_on_no_listeners() {
        let signal = Signal::new();
        signal.emit(&"test");
    }

    #[test]
    fn mixed_types_in_callbacks() {
        let signal = Signal::new();
        let int_state = Rc::new(RefCell::new(0));
        let string_state = Rc::new(RefCell::new(String::new()));

        let int_clone = Rc::clone(&int_state);
        signal.connect(move |data| {
            if let Some(v) = data.downcast_ref::<i32>() {
                *int_clone.borrow_mut() += v;
            }
        });

        let str_clone = Rc::clone(&string_state);
        signal.connect(move |data| {
            if let Some(s) = data.downcast_ref::<&str>() {
                *str_clone.borrow_mut() = s.to_string();
            }
        });

        signal.emit(&10);
        signal.emit(&"text");
        signal.emit(&20);

        assert_eq!(*int_state.borrow(), 30);
        assert_eq!(*string_state.borrow(), "text");
    }
}
