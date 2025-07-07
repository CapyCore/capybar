use std::any::Any;

type Callback = Box<dyn Fn(&dyn Any)>;

/// Signal is a mechanism that allows communication between different components
///
/// Signal can be treated as a tunnel. You can pass values through the tunnel and
/// have functions that react to those values. This allows to split heavy procceses
/// from displaing the results. This is done by alowing to bind function that can
/// react to a passed data.
///
/// Downcasting and validating data is left to user. You are only provided with 
/// `dyn Any`.
///
/// Whenever signal emit is called all the functions connected to a signal are called.
///
/// In capybar signals are stored inside an [Environment](crate::root::Environment)
/// in a `signals` [HashMap](std::collections::HashMap)
///
/// # Examples
///
/// ```
/// let mut signal = Signal::new();
/// let last_value = Rc::new(RefCell::new(None));
/// let last_value_clone = Rc::clone(&last_value);
///
/// signal.connect(move |data| {
///     if let Some(value) = data.downcast_ref::<i32>() {
///         *last_value_clone.borrow_mut() = Some(*value);
///     }
/// });
///
/// //...
///
/// signal.emit(&42i32);
/// ```
pub struct Signal {
    listeners: Vec<Callback>,
}

impl Signal {
    pub fn new() -> Self {
        Signal {
            listeners: Vec::new(),
        }
    }

    pub fn connect<F>(&mut self, callback: F)
    where
        F: Fn(&dyn Any) + 'static,
    {
        self.listeners.push(Box::new(callback));
    }

    pub fn emit<T: Any>(&self, value: &T) {
        for callback in &self.listeners {
            callback(value);
        }
    }
}
