//! Reusable UI components built on top of egui.
//!
//! Every component is a thin wrapper over egui primitives that bakes
//! in our design tokens — colors, spacing, radii, shadows — so widget
//! code can stop hand-rolling the same `Frame` and `RichText` chains
//! across a dozen files.
//!
//! See [`STYLE_GUIDE.md`](../../../STYLE_GUIDE.md) §3 for the contract
//! of each component.

#![allow(dead_code)] // some components are used after the sweep; others land later

pub mod banner;
pub mod buttons;
pub mod card;
pub mod empty_state;
pub mod hint;
pub mod list_row;
pub mod modal_frame;
pub mod pill;
pub mod search_input;
pub mod section;

pub use banner::Banner;
pub use buttons::{ghost_button, icon_button, primary_button};
pub use card::Card;
pub use empty_state::empty_state;
pub use hint::hint_row;
pub use list_row::{list_row, RowState};
pub use modal_frame::modal_frame;
pub use pill::{neutral_pill, status_pill, StatusKind};
pub use search_input::search_input;
pub use section::{section, section_header};
