use rstar::RTree;

/// Describes a visualization that searches the R* tree and save the result as csv
/// The result can be used to plot with python
/// The generic type T is anything the search function returns
pub trait Search<T> {
    /// Search the tree and output it to a file (actually stdout)
    /// The python script can read the result and plot it
    fn search_to_file(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]);

    /// The function that searches the R* tree.
    /// The stations are stored in the tree. For every population point
    /// in pp_lines, the function searches for the nearest neighbours within
    /// max_distance. It returns anything the visualization needs, such as...
    ///
    /// cumulative_props.rs:
    /// - the population of the point if a station within max_distance is found
    ///
    /// stations_within_pp.rs:
    /// - the number of stations within max_distance of any population point
    ///
    /// quadrants.rs:
    /// - the population of every point and the number of stations within
    ///   max_distance of it
    fn search(
        tree: &RTree<(f64, f64)>,
        pp_lines: &[&str],
        max_distance: f64,
    ) -> T;
}

/// Describes a visualization that searches the R* tree and plots the result in rust
/// It requires the visualization to implement Search, as it relies on the search
/// function. The result of the search function can be anything (U), as long
/// as it can be transformed into T
pub trait Plot<T, U>: Search<U> {
    /// Search the tree and immediately plot the results with rust.
    /// Use when python cannot handle the amount of data
    fn search_to_plot(&self, tree: &RTree<(f64, f64)>, pp_lines: &[&str]);

    /// The function that does the plotting
    fn plot(&self, data: T) -> Result<(), Box<dyn std::error::Error>>;
}

