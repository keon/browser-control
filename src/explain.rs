//! Educational metadata for common bit hacks. Feature `explain`.

/// Static description of a bit operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Explanation {
    /// Operation name.
    pub name: &'static str,
    /// Closed-form formula.
    pub formula: &'static str,
    /// Short prose intuition.
    pub intuition: &'static str,
    /// Caveats and pitfalls.
    pub caveats: &'static [&'static str],
}

macro_rules! e { ($fn:ident, $name:literal, $formula:literal, $intuition:literal, $caveats:expr) => {
    #[doc = concat!("Explains `", $name, "`: `", $formula, "`.")]
    pub const fn $fn() -> Explanation { Explanation { name: $name, formula: $formula, intuition: $intuition, caveats: $caveats } }
}; }

e!(isolate_lowest_set_bit,  "isolate_lowest_set_bit", "x & -x",
   "Two's-complement negation flips bits at and above the lowest set bit; AND preserves only the lowest set bit.",
   &["returns 0 when x == 0"]);
e!(clear_lowest_set_bit,    "clear_lowest_set_bit",   "x & (x - 1)",
   "Subtracting 1 flips all bits at or below the lowest set bit; AND clears it.",
   &["returns 0 when x == 0"]);
e!(is_power_of_two,         "is_power_of_two",        "x != 0 && (x & (x - 1)) == 0",
   "A power of two has exactly one set bit; clearing it yields zero.",
   &["0 is not considered a power of two"]);
e!(isolate_lowest_zero_bit, "isolate_lowest_zero_bit","!x & -(!x)",
   "Bitwise NOT flips zeros to ones; isolating the lowest set bit of !x finds the lowest zero of x.",
   &["returns 0 when x is all ones"]);
e!(set_lowest_zero_bit,     "set_lowest_zero_bit",    "x | (x + 1)",
   "x + 1 flips the trailing run of ones plus the lowest zero; OR sets that zero.",
   &["returns x unchanged when x is all ones"]);
e!(align_up,                "align::up",              "(value + align - 1) & !(align - 1)",
   "Bias upward by `align - 1`, then mask off the low bits.",
   &["value + align - 1 can overflow at the top of the address space"]);
e!(align_down,              "align::down",            "value & !(align - 1)",
   "Mask off the low bits to round down to a multiple of `align`.",
   &[]);
e!(padding_needed,          "align::padding",         "(-value) & (align - 1)",
   "The two's-complement negation of value modulo `align` is the distance to the next multiple.",
   &[]);
e!(submasks,                "submasks",               "s_{n+1} = (s_n - 1) & mask, s_0 = mask",
   "Subtract 1, mask back to the mask's bits; visits every submask in descending numeric order.",
   &["visits 2^popcount(mask) values"]);
