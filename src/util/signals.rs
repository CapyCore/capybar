use std::any::Any;

type Callback = Box<dyn Fn(&dyn Any)>;

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
