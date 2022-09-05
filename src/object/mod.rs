pub use helper::decorate;

pub trait Object: Clone + Send + Sync + 'static {
    fn uid(&self) -> &str;
    fn version(&self) -> u64;
    fn kind(&self) -> &str;
}
