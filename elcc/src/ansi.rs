//! ANSI styling.

use std::fmt;
use yansi::Painted;

/// Dummy data type for an empty display output.
#[derive(Debug, Clone, Copy)]
pub struct Empty;

impl fmt::Display for Empty {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        Ok(())
    }
}

/// Pure painter.
pub type Painter = Painted<Empty>;

/// Painter that does nothing.
pub const NOP: Painter = Painted::new(Empty);

/// Painter that lingers.
pub const LINGER: Painter = NOP.linger();

/// Painter that makes the output dim.
pub const DIM: Painter = LINGER.dim();

/// Painter that resets the style.
pub const RESET: Painter = NOP.resetting();
