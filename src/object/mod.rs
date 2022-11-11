pub use helper::decorate;

pub trait Object: Clone + Send + Sync + 'static {
    fn uid(&self) -> &str;
    fn version(&self) -> u64;
    fn kind(&self) -> &str;

    fn set_uid(&mut self, id: &str);
    fn set_version(&mut self, version: u64);
}
