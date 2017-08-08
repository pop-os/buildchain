/// The location to run the build
pub enum Location {
    /// Run the build locally
    Local,
    /// Run the build on the specified remote server
    Remote(String),
}
