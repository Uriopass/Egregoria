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

pub type SparseGrid<O> = Grid<O>;

pub type SparseShapeGrid<O, S> = ShapeGrid<O, S>;
