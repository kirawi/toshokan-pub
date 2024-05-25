pub struct Series {}

pub trait Source {
    /// Get a list of all the series tracked by a source.
    fn series(&self) -> Series;
}
