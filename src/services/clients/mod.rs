//! Current module describes all of capybars clients. Different compositors handle some stuff
//! differently. All of the unique behaviours is described here.

use super::Service;

#[cfg(feature = "hyprland")]
pub mod hyprland;

#[allow(dead_code)]
#[cfg(feature = "keyboard")]
trait KeyboardTrait: Service {}

#[cfg(feature = "keyboard+hyprland")]
pub use hyprland::keyboard::Keyboard;
