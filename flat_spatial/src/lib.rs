#![deny(
    rustdoc::all,
    rust_2018_compatibility,
    rust_2018_idioms,
    nonstandard_style,
    unused,
    future_incompatible,
    unused_extern_crates,
    clippy::all,
    clippy::doc_markdown,
    clippy::wildcard_imports
)]
#![allow(
    clippy::collapsible_else_if,
    clippy::manual_range_contains,
    clippy::unreadable_literal,
    clippy::unseparated_literal_suffix,
    clippy::blocks_in_if_conditions,
    clippy::upper_case_acronyms,
    clippy::must_use_candidate,
    missing_copy_implementations,
    missing_debug_implementations
)]
//!
//! `flat_spatial` is a crate dedicated to spatial partitioning structures that are not based on trees
//! (which are recursive) but on simple flat structures such as grids.
//!
//! Both `DenseGrid` and `SparseGrid` partition the space using cells of user defined width.
//! `DenseGrid` uses a Vec of cells and `SparseGrid` a `FastMap` (so cells are lazily allocated).
//!

pub mod cell;
pub mod grid;
pub mod shapegrid;
pub mod storage;

pub use grid::Grid;
pub use shapegrid::ShapeGrid;

use storage::DenseStorage;
use storage::SparseStorage;

pub type DenseGrid<O> = Grid<O, DenseStorage<cell::GridCell>>;
pub type SparseGrid<O> = Grid<O, SparseStorage<cell::GridCell>>;

pub type DenseShapeGrid<O, S> = ShapeGrid<O, S, DenseStorage<cell::ShapeGridCell>>;
pub type SparseShapeGrid<O, S> = ShapeGrid<O, S, SparseStorage<cell::ShapeGridCell>>;
