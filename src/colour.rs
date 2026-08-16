// ============================================================================
// COLOUR UTILITIES
// ============================================================================

use owo_colors::{FgDynColorDisplay, OwoColorize, Rgb};
use owo_colors::styles::{BoldDisplay, DimDisplay, ItalicDisplay};

/// Shorthand trait for truecolour formatting
///
/// Provides a shorter `.tc()` method as an alias for `.truecolor()` from owo_colors.
/// This reduces verbosity when specifying RGB colours for terminal output.
///
/// Example:
/// ```rust
/// "text".tc(255, 0, 0)  // Red text
/// "text".tc(0, 255, 0)  // Green text
/// ```
pub trait Colour {
    fn tc(&self, r: u8, g: u8, b: u8) -> FgDynColorDisplay<'_, Rgb, Self>;
    fn c(&self, rgb: (u8, u8, u8)) -> FgDynColorDisplay<'_, Rgb, Self>;
    // bold
    fn b(&self) -> BoldDisplay<'_, Self>;
    // dim
    fn d(&self) -> DimDisplay<'_, Self>;
    // italic
    fn i(&self) -> ItalicDisplay<'_, Self>;
}

// Implement it for all types that already implement OwoColorize
impl<T: OwoColorize> Colour for T {
    #[inline(always)]
    fn tc(&self, r: u8, g: u8, b: u8) -> FgDynColorDisplay<'_, Rgb, Self> {
        self.truecolor(r, g, b)
    }

    #[inline(always)]
    fn c(&self, (r, g, b): (u8, u8, u8)) -> FgDynColorDisplay<'_, Rgb, Self> {
        self.truecolor(r, g, b)
    }

    #[inline(always)]
    fn b(&self) -> BoldDisplay<'_, Self> {
        self.bold()
    }

    #[inline(always)]
    fn d(&self) -> DimDisplay<'_, Self> {
        self.dimmed()
    }

    #[inline(always)]
    fn i(&self) -> ItalicDisplay<'_, Self> {
        self.italic()
    }
}

pub const RED: (u8, u8, u8) = (200, 0, 0);
pub const ORANGE: (u8, u8, u8) = (200, 150, 0);
pub const GREEN: (u8, u8, u8) = (0, 200, 0);
pub const CYAN: (u8, u8, u8) = (0, 200, 200);
pub const YELLOW: (u8, u8, u8) = (200, 200, 0);
pub const MAGENTA: (u8, u8, u8) = (200, 0, 200);
