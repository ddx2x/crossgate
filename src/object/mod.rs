pub use helper::metadata;

pub trait Object: Clone + Send + Sync + 'static {
    fn uid(&self) -> &str;
    fn version(&self) -> u64;
    fn kind(&self) -> &str;

    fn update_uid(&mut self, id: &str);
    fn update_version(&mut self, version: u64);
}
