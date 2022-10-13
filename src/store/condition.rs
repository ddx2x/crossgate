use super::Filter;

#[derive(Clone, Debug)]
pub struct Condition<T: Filter> {
    pub(crate) db: String,
    pub(crate) table: String,
    pub(crate) page: usize,
    pub(crate) page_size: usize,
    pub(crate) sorts: Vec<String>,
    pub(crate) fields: Vec<String>,
    pub(crate) filter: T,
}

impl<T> Condition<T>
where
    T: Filter,
{
    pub fn new(t: T) -> Self {
        Self {
            db: Default::default(),
            table: Default::default(),
            page: 1,
            page_size: 10,
            sorts: Default::default(),
            fields: Default::default(),
            filter: t,
        }
    }
    pub fn with_db(&mut self, db: &str) -> &mut Condition<T> {
        self.db = db.to_string();
        self
    }

    pub fn with_table(&mut self, table: &str) -> &mut Condition<T> {
        self.table = table.to_string();
        self
    }

    pub fn with_sort(&mut self, sorts: &[&str]) -> &mut Condition<T> {
        let sort = sorts
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        self.sorts = sort;
        self
    }

    pub fn with_fields(&mut self, fields: &[&str]) -> &mut Condition<T> {
        let fields = fields
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        self.fields = fields;
        self
    }

    pub fn with_page(&mut self, page: usize, page_size: usize) -> &mut Condition<T> {
        self.page = page;
        self.page_size = page_size;
        self
    }

    pub fn wheres(&mut self, input: &str) -> anyhow::Result<()> {
        self.filter.parse(input)?;
        Ok(())
    }

    pub fn sorts<'a>(&'a self) -> &'a [String] {
        self.sorts.as_slice()
    }
}

#[cfg(test)]
mod test {
    use crate::store::new_mongo_condition;

    #[test]
    fn test_parse_cond() {
        let mut cond = new_mongo_condition();
        match cond.wheres("a=1&&b=2||c=1&&b=2&&abc='abc21'") {
            Ok(_) => println!("{:?}", cond),
            Err(e) => panic!("{}", e),
        }
    }
}
