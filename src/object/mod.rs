pub use helper::decorate;

pub trait Object: Clone + Send + Sync + 'static {
    fn uid(&self) -> &str;
    fn version(&self) -> u64;
    fn kind(&self) -> &str;
    fn generate(&mut self, f: fn() -> String); // 自动生成uuid函数
    fn get_version(&mut self);
}
