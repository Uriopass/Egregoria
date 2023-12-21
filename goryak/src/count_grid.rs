use std::cell::RefCell;
use yakui_core::geometry::{Constraints, Vec2};
use yakui_core::widget::{LayoutContext, Widget};
use yakui_core::{CrossAxisAlignment, Direction, MainAxisAlignment, MainAxisSize, Response};

use yakui_widgets::util::widget_children;

/// Defines alignment for items within a container's main axis when there is space left.
///
/// This occurs in a Grid when items of the same row are bigger than one self.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MainAxisAlignItems {
    /// Align item to the beginning of the cell main axis.
    ///
    /// For a left-to-right grid, this is the left side of the cell.
    ///
    /// For a top-down grid, this is the top of the cell.
    Start,

    /// Align items to the center of the cell's main axis.
    Center,

    /// Align items to the end of the cell's main axis.
    ///
    /// For a left-to-right list, this is the right side of the cell.
    ///
    /// For a top-down list, this is the bottom of the cell.
    End,

    /// Stretch items to fill the maximum size of the cell's main axis.
    Stretch,
}

/**
CountGrid lays out its children such as all cells within the same column have the same width, and
all cells within the same row have the same height.

The children should be provided in cross-axis-major order.
For example, if you want a 2x3 column-based grid, you should provide the children in this order:
```text
0 1
2 3
4 5
```

The grid tries to replicate the same layout logic as a List.
A n x 1 grid should be almost equivalent to a List for non-flex content.

Check the count_grid example to see it in action with different alignments and sizes.

Responds with [CountGridResponse].
 */
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CountGrid {
    pub direction: Direction,
    pub cross_axis_count: usize,
    pub main_axis_alignment: MainAxisAlignment,
    pub main_axis_size: MainAxisSize,
    pub main_axis_align_items: MainAxisAlignItems,
    pub cross_axis_alignment: CrossAxisAlignment,
}

impl CountGrid {
    /// The children will be laid out in a grid with the given number of columns.
    /// They should be provided in row-major order.
    pub fn col(n_columns: usize) -> Self {
        Self {
            direction: Direction::Down,
            cross_axis_count: n_columns,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            main_axis_align_items: MainAxisAlignItems::Start,
        }
    }

    /// The children will be laid out in a grid with the given number of rows.
    /// They should be provided in column-major order.
    pub fn row(n_rows: usize) -> Self {
        Self {
            direction: Direction::Right,
            cross_axis_count: n_rows,
            main_axis_size: MainAxisSize::Max,
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Start,
            main_axis_align_items: MainAxisAlignItems::Start,
        }
    }

    pub fn cross_axis_aligment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    pub fn main_axis_aligment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    pub fn main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }

    pub fn main_axis_align_items(mut self, items: MainAxisAlignItems) -> Self {
        self.main_axis_align_items = items;
        self
    }

    /// The children will be laid out in a grid with the given number of columns/rows.
    /// They should be provided in cross-axis-major order.
    /// For example, if you want a 2x3 column-based grid, you should provide the children in this order:
    /// ```text
    /// 0 1
    /// 2 3
    /// 4 5
    /// ```
    pub fn show<F: FnOnce()>(self, children: F) -> Response<CountGridResponse> {
        widget_children::<CountGridWidget, F>(children, self)
    }
}

#[derive(Debug)]
pub struct CountGridWidget {
    props: CountGrid,
    max_sizes: RefCell<Vec<f32>>, // cache max_sizes vector to avoid reallocating every frame
}

pub type CountGridResponse = ();

impl Widget for CountGridWidget {
    type Props<'a> = CountGrid;
    type Response = CountGridResponse;

    fn new() -> Self {
        Self {
            props: CountGrid::col(0),
            max_sizes: RefCell::new(vec![]),
        }
    }

    fn update(&mut self, props: Self::Props<'_>) -> Self::Response {
        self.props = props;
    }

    fn layout(&self, mut ctx: LayoutContext<'_>, input: Constraints) -> Vec2 {
        let node = ctx.dom.get_current();

        let n_cross = self.props.cross_axis_count;
        let direction = self.props.direction;

        // Pad the number of children to be a multiple of cross_n (if not already the case)
        let n = node.children.len();
        let n_cells = n + (n_cross - n % n_cross) % n_cross;

        let n_main = n_cells / n_cross;

        // Calculate cell constraints
        // In general, to get cell constraint we divide the input constraints by the number of cells
        // in each axis

        let cell_cross_max = direction.get_cross_axis(input.max) / n_cross as f32;
        let cell_cross_min = match self.props.cross_axis_alignment {
            // If stretch, the cells will be as wide as possible
            CrossAxisAlignment::Stretch => cell_cross_max,
            _ => 0.0,
        };

        // Same logic as for lists, we cannot allow going infinitely far in the main axis
        let mut total_main_max = direction.get_main_axis(input.max);
        if total_main_max.is_infinite() {
            total_main_max = direction.get_main_axis(input.min);
        };

        let cell_main_max = total_main_max / n_main as f32;
        let cell_main_min = match self.props.main_axis_align_items {
            MainAxisAlignItems::Stretch => cell_main_max,
            _ => 0.0,
        };

        let cell_constraint = Constraints {
            min: direction.vec2(cell_main_min, cell_cross_min),
            max: direction.vec2(cell_main_max, cell_cross_max),
        };

        // max_sizes holds the maximum size on cross axis and main axis
        // its layout is:
        // 0 ... n_cross - 1 ... n_cross .. (n_cross + n_main)
        // where each element is the maximum size of each row/column in each axis
        // it is used later to calculate where each cell should go
        // it is put into a RefCell to avoid reallocating every frame
        let mut max_sizes = std::mem::take(&mut *self.max_sizes.borrow_mut());
        max_sizes.resize(n_cross + n_main, 0.0);

        // dispatch layout and find the maximum size of each row/column
        for (i, &child_id) in node.children.iter().enumerate() {
            let size = ctx.calculate_layout(child_id, cell_constraint);

            let main_id = i / n_cross;
            let cross_id = i % n_cross;

            let main_size = direction.get_main_axis(size);
            let cross_size = direction.get_cross_axis(size);

            max_sizes[n_cross + main_id] = max_sizes[n_cross + main_id].max(main_size);
            max_sizes[cross_id] = max_sizes[cross_id].max(cross_size);
        }

        // We keep track of the final size of each axis to apply alignment later + total grid size
        // + set the positions without more allocations
        let mut total_main_size: f32 = 0.0;
        let mut max_total_cross_size: f32 = 0.0;

        // Set the positions without caring for alignment for now (as if alignment was Start, Start)
        for main_axis_id in 0..n_main {
            let cross_line_slice = &node.children
                [main_axis_id * n_cross..((main_axis_id + 1) * n_cross).min(node.children.len())];

            // We keep track of cross axis size to set positions of the cross-axis line
            let mut total_cross_size = 0.0;
            for (cross_axis_id, &child_id) in cross_line_slice.iter().enumerate() {
                let layout = ctx.layout.get_mut(child_id).unwrap();

                let cross_axis_size = match self.props.cross_axis_alignment {
                    CrossAxisAlignment::Stretch => cell_cross_max,
                    _ => max_sizes[cross_axis_id],
                };

                let pos = direction.vec2(total_main_size, total_cross_size);
                layout.rect.set_pos(pos);

                total_cross_size += cross_axis_size;
                max_total_cross_size = max_total_cross_size.max(total_cross_size);
            }

            total_main_size += max_sizes[n_cross + main_axis_id];
        }

        // Calculate offset needed for alignment
        let mut offset_main_global = match self.props.main_axis_alignment {
            MainAxisAlignment::Start => 0.0,
            MainAxisAlignment::Center => ((total_main_max - total_main_size) / 2.0).max(0.0),
            MainAxisAlignment::End => (total_main_max - total_main_size).max(0.0),
            other => unimplemented!("MainAxisAlignment::{other:?}"),
        };
        offset_main_global = match self.props.main_axis_size {
            MainAxisSize::Max => offset_main_global,
            MainAxisSize::Min => 0.0,
            other => unimplemented!("MainAxisSize::{other:?}"),
        };

        // only used in case the widget total cross is less than the minimum cross axis
        let offset_cross_global = match self.props.cross_axis_alignment {
            CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
            CrossAxisAlignment::Center => {
                ((direction.get_cross_axis(input.min) - max_total_cross_size) / 2.0).max(0.0)
            }
            CrossAxisAlignment::End => {
                (direction.get_cross_axis(input.min) - max_total_cross_size).max(0.0)
            }
            other => unimplemented!("CrossAxisAlignment::{other:?}"),
        };

        // Apply alignment by offsetting all children
        for (i, &child_id) in node.children.iter().enumerate() {
            let cross_id = i % n_cross;
            let main_id = i / n_cross;

            let layout = ctx.layout.get_mut(child_id).unwrap();

            let child_cross_size = direction.get_cross_axis(layout.rect.size());
            let cell_cross_size = match self.props.cross_axis_alignment {
                CrossAxisAlignment::Stretch => cell_cross_max,
                _ => max_sizes[cross_id],
            };
            let offset_cross = match self.props.cross_axis_alignment {
                CrossAxisAlignment::Start | CrossAxisAlignment::Stretch => 0.0,
                CrossAxisAlignment::Center => ((cell_cross_size - child_cross_size) / 2.0).max(0.0),
                CrossAxisAlignment::End => (cell_cross_size - child_cross_size).max(0.0),
                other => unimplemented!("CrossAxisAlignment::{other:?}"),
            };

            let child_main_size = direction.get_main_axis(layout.rect.size());
            let cell_main_size = match self.props.main_axis_align_items {
                MainAxisAlignItems::Start | MainAxisAlignItems::Stretch => cell_main_max,
                _ => max_sizes[n_cross + main_id],
            };
            let offset_main = match self.props.main_axis_align_items {
                MainAxisAlignItems::Start | MainAxisAlignItems::Stretch => 0.0,
                MainAxisAlignItems::Center => ((cell_main_size - child_main_size) / 2.0).max(0.0),
                MainAxisAlignItems::End => (cell_main_size - child_main_size).max(0.0),
                other => unimplemented!("MainAxisAlignItems::{other:?}"),
            };

            let offset_pos = layout.rect.pos()
                + direction.vec2(
                    offset_main_global + offset_main,
                    offset_cross_global + offset_cross,
                );
            layout.rect.set_pos(offset_pos);
        }

        // Put max_sizes back to be reused
        max_sizes.clear();
        let _ = std::mem::replace(&mut *self.max_sizes.borrow_mut(), max_sizes);

        // Figure out the final size of the grid
        let cross_grid_size = match self.props.cross_axis_alignment {
            CrossAxisAlignment::Stretch => direction.get_cross_axis(input.max),
            _ => max_total_cross_size,
        };
        let main_grid_size = match self.props.main_axis_size {
            MainAxisSize::Max => total_main_max,
            MainAxisSize::Min => total_main_size,
            other => unimplemented!("MainAxisSize::{other:?}"),
        };

        direction.vec2(main_grid_size, cross_grid_size)
    }
}
