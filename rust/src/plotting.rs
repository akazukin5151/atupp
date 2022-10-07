use plotters::coord::types::{RangedCoordf64, RangedCoordi32};
use plotters::coord::Shift;
use plotters::prelude::*;

// adapted from my previous project train-passenger-distribution
pub type Chart<'a, 'b> = ChartContext<
    'a,
    BitMapBackend<'b>,
    Cartesian2d<RangedCoordf64, RangedCoordi32>,
>;

pub fn plot_vline(
    root: &DrawingArea<BitMapBackend, Shift>,
    chart: &Chart,
    x_value: f64,
    modifier: i32,
    top_y: i32,
    stroke: ShapeStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    let drawing_area = chart.plotting_area();
    let mapped = drawing_area.map_coordinate(&(x_value, 0));
    let p: PathElement<(i32, i32)> = PathElement::new(
        [(mapped.0, mapped.1 - modifier), (mapped.0, top_y)],
        stroke,
    );
    root.draw(&p)?;
    Ok(())
}

pub fn plot_hline(
    root: &DrawingArea<BitMapBackend, Shift>,
    chart: &Chart,
    y_value: i32,
    modifier: i32,
    left_x: f64,
    stroke: ShapeStyle,
) -> Result<(), Box<dyn std::error::Error>> {
    let drawing_area = chart.plotting_area();
    let mapped = drawing_area.map_coordinate(&(0., y_value));
    let end = drawing_area.map_coordinate(&(left_x, y_value));
    let p: PathElement<(i32, i32)> = PathElement::new(
        [(mapped.0, mapped.1 - modifier), (end.0, end.1)],
        stroke,
    );
    root.draw(&p)?;
    Ok(())
}
