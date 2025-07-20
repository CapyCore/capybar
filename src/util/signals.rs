use std::{
    any::Any,
    cell::{Ref, RefCell},
};

type Callback = Box<dyn Fn(&dyn Any)>;

/// Reactive communication channel for decoupled component interaction
///
/// Signals implement a publish-subscribe pattern where:
/// - Publishers emit values through [emit](Signal::emit) or [emit_unclonable](Signal::emit_unclonable)
/// - Subscribers register callbacks via [connect](Signal::connect)
///
/// ### Core Features
/// - **Type-erased values**: All emitted values are passed as `&dyn Any`
/// - **Value history**: Optionally stores last emitted value (see [emit](Signal::emit))
/// - **Immediate callback invocation**: New connections receive current value immediately
///
/// ### Behavior Details
/// - **Downcasting responsibility**: Receivers must validate and downcast values
/// - **Callback persistence**: Handlers remain registered until signal destruction
///
/// ### Usage Notes
/// - Prefer `emit` for cloneable types requiring history
/// - Use `emit_unclonable` for non-cloneable types or when history isn't needed
/// - In capybar, signals are stored in an [Environment](crate::root::Environment)'s
///   `signals` [HashMap](std::collections::HashMap)
///
/// # Examples
/// ```
/// use capybar::util::signals::Signal;
/// use std::{cell::RefCell, rc::Rc};
///
/// let signal = Signal::new();
/// let tracker = Rc::new(RefCell::new(0));
///
/// // Connect callback that servicees i32 values
/// let track = Rc::clone(&tracker);
/// signal.connect(move |data| {
///     if let Some(num) = data.downcast_ref::<i32>() {
///         *track.borrow_mut() = *num;
///     }
/// });
///
/// // Emit value to all connected callbacks
/// signal.emit(&42i32);
/// assert_eq!(*tracker.borrow(), 42);
/// ```
#[derive(Default)]
pub struct Signal {
    listeners: RefCell<Vec<Callback>>,
    last_value: RefCell<Option<Box<dyn Any>>>,
}

impl Signal {
    /// Creates a new, empty Signal instance
    pub fn new() -> Self {
        Signal {
            listeners: RefCell::new(Vec::new()),
            last_value: RefCell::new(None),
        }
    }
    /// Registers a callback to be invoked on signal emissions
    ///
    /// The callback will be immediately invoked with the current `last_value`
    /// if one exists. All registered callbacks are invoked when [emit](Signal::emit)
    /// is called.
    ///
    /// # Arguments
    /// * `callback` - Handler function that receives emitted data as `&dyn Any`
    ///
    /// Note: Callbacks persist until the Signal is dropped
    pub fn connect<F>(&self, callback: F)
    where
        F: Fn(&dyn Any) + 'static,
    {
        if let Some(value) = &*self.last_value.borrow() {
            callback(&**value);
        }

        self.listeners.borrow_mut().push(Box::new(callback));
    }

    /// Emits a value to all connected callbacks
    ///
    /// This operation:
    /// 1. Clones the value (must implement [Any] + [Clone])
    /// 2. Stores the cloned value as the new `last_value`
    /// 3. Invokes all callbacks with a reference to the original value
    ///
    /// Prefer this over [emit_unclonable](Signal::emit_unclonable) when:
    /// - You need value history tracking
    /// - Your type is cheap to clone
    pub fn emit<T: Any + Clone>(&self, value: &T) {
        let cloned = (*value).clone();
        *self.last_value.borrow_mut() = Some(Box::new(cloned));
        for callback in &*self.listeners.borrow_mut() {
            callback(value);
        }
    }

    /// Emits a value without storing or cloning it
    ///
    /// Unlike [emit](Signal::emit):
    /// - Doesn't update `last_value`
    /// - Doesn't require [Clone] implementation
    /// - Slightly more efficient for non-cloneable types
    ///
    /// Use when:
    /// - You don't need value history
    /// - The value can't be cloned
    /// - Callbacks don't need persistent access to the value
    pub fn emit_unclonable<T: Any>(&self, value: &T) {
        for callback in &*self.listeners.borrow_mut() {
            callback(value);
        }
    }

    /// Returns a read-only reference to the internal last_value storage
    ///
    /// Example usage:
    /// ```
    /// use capybar::util::signals::Signal;
    ///
    /// let signal = Signal::new();
    /// signal.emit(&42i32);
    /// if let Some(value) = &*signal.last_value_ref() {
    ///     if let Some(num) = value.downcast_ref::<i32>() {
    ///         assert_eq!(*num, 42);
    ///     }
    /// };
    /// ```
    pub fn last_value_ref(&self) -> Ref<'_, Option<Box<dyn Any>>> {
        self.last_value.borrow()
    }

    /// Processes the last value through a callback function
    ///
    /// Example usage:
    /// ```
    /// use capybar::util::signals::Signal;
    ///
    /// let signal = Signal::new();
    /// signal.emit(&42i32);
    /// signal.with_last_value(|any| {
    ///     if let Some(num) = any.and_then(|a| a.downcast_ref::<i32>()) {
    ///         assert_eq!(*num, 42);
    ///     }
    /// });
    /// ```
    pub fn with_last_value<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Option<&dyn Any>) -> R,
    {
        let last_value = self.last_value_ref();
        let any_ref = last_value.as_ref().map(|boxed| &**boxed as &dyn Any);
        f(any_ref)
    }

    /// Retrieves a cloned copy of the last value if available and of type T
    ///
    /// Example usage:
    /// ```
    /// use capybar::util::signals::Signal;
    ///
    /// let signal = Signal::new();
    /// signal.emit(&42i32);
    /// if let Some(num) = signal.get_last_value_cloned::<i32>() {
    ///     assert_eq!(num, 42);
    /// }
    /// ```
    pub fn get_last_value_cloned<T: Any + Clone>(&self) -> Option<T> {
        self.with_last_value(|opt| opt.and_then(|any| any.downcast_ref::<T>().cloned()))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SignalNames {
    Keyboard,
    Custom(String),
}
