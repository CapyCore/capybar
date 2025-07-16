#[cfg(test)]
mod tests {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use capybar::util::signals::Signal;

    #[test]
    fn initial_state() {
        let signal = Signal::new();
        assert!(signal.last_value_ref().is_none());
    }

    #[test]
    fn emit_store_last_value() {
        let signal = Signal::new();
        signal.emit(&42i32);
        assert_eq!(signal.get_last_value_cloned::<i32>(), Some(42));
    }

    #[test]
    fn emit_unclonable_does_not_store() {
        let signal = Signal::new();
        signal.emit(&100u8);
        signal.emit_unclonable(&200u8);
        assert_eq!(signal.get_last_value_cloned::<u8>(), Some(100));
    }
    #[test]

    fn connect_triggers_immediately() {
        let signal = Signal::new();
        signal.emit(&"initial");
        let triggered = Rc::new(Cell::new(false));
        let trigger_clone = Rc::clone(&triggered);
        signal.connect(move |data| {
            if data.downcast_ref::<&str>().is_some() {
                trigger_clone.set(true);
            }
        });
        assert!(triggered.get());
    }

    #[test]
    fn no_immediate_trigger_without_last_value() {
        let signal = Signal::new();
        let triggered = Rc::new(Cell::new(false));
        let trigger_clone = Rc::clone(&triggered);
        signal.connect(move |_| trigger_clone.set(true));
        assert!(!triggered.get());
    }

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

        signal.emit_unclonable(&42i32);
        assert_eq!(*last_value.borrow(), Some(42));

        signal.emit_unclonable(&100i32);
        assert_eq!(*last_value.borrow(), Some(100));
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn recursive_emit_panics() {
        let signal = Signal::new();
        let signal_clone = Rc::new(signal);
        let weak = Rc::downgrade(&signal_clone);
        signal_clone.connect(move |_| {
            if let Some(s) = weak.upgrade() {
                s.emit(&"recursive");
            }
        });
        signal_clone.emit(&"trigger");
    }

    #[test]
    fn type_erasure_handling() {
        let signal = Signal::new();
        signal.emit(&1i32);
        signal.emit(&"string");
        let result = signal.get_last_value_cloned::<&str>();
        assert_eq!(result, Some("string"));
    }

    #[test]
    fn with_last_value_helper() {
        let signal = Signal::new();
        signal.emit(&999u64);
        signal.with_last_value(|opt| {
            assert_eq!(
                opt.and_then(|v| v.downcast_ref::<u64>()).copied(),
                Some(999)
            );
        });
    }

    #[test]
    fn non_clone_type_emission() {
        struct NonClone(i32);
        let signal = Signal::new();
        signal.emit_unclonable(&NonClone(42));
        // Verify emission occurred via callback
        let received = Rc::new(Cell::new(None));
        let recv_clone = Rc::clone(&received);
        signal.connect(move |data| {
            if let Some(nc) = data.downcast_ref::<NonClone>() {
                recv_clone.set(Some(nc.0));
            }
        });
        signal.emit_unclonable(&NonClone(100));
        assert_eq!(received.get(), Some(100));
    }

    #[test]
    fn get_last_value_wrong_type() {
        let signal = Signal::new();
        signal.emit(&5i16);
        assert!(signal.get_last_value_cloned::<i32>().is_none());
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

        signal.emit_unclonable(&42i32);
        signal.emit_unclonable(&"ignore");
        assert_eq!(*state.borrow(), 84);
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
        signal.emit_unclonable(&"not a bool");
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

        signal.emit_unclonable(&1i32);
        signal.emit_unclonable(&2i32);
        signal.emit_unclonable(&3i32);

        assert_eq!(*value.borrow(), 3);
    }

    #[test]
    fn no_panic_on_no_listeners() {
        let signal = Signal::new();
        signal.emit(&"test");
        signal.emit_unclonable(&42i32);
    }

    #[test]
    fn emit_many_listeners() {
        let signal = Signal::new();
        let count = Rc::new(RefCell::new(0));

        // Add 1000 listeners
        for _ in 0..1000 {
            let count_clone = Rc::clone(&count);
            signal.connect(move |data| {
                if data.downcast_ref::<i32>().is_some() {
                    *count_clone.borrow_mut() += 1;
                }
            });
        }

        signal.emit(&1);
        assert_eq!(*count.borrow(), 1000);

        signal.emit_unclonable(&1);
        assert_eq!(*count.borrow(), 2000);
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

        signal.emit_unclonable(&10);
        signal.emit_unclonable(&"test");
        signal.emit_unclonable(&20);

        assert_eq!(*int_state.borrow(), 60);
        assert_eq!(*string_state.borrow(), "test");
    }
}
