use super::Filter;

#[derive(
    PartialEq, Debug, Copy, Clone, serde_repr::Serialize_repr, serde_repr::Deserialize_repr,
)]
#[repr(u8)]
pub enum SortDirection {
    Ascending = 1,
    Descending = 2,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Sort {
    pub field: String,
    pub order: SortDirection,
}

#[derive(Clone, Debug)]
pub struct Condition<T: Filter> {
    pub(crate) db: String,
    pub(crate) table: String,
    pub(crate) page: usize,
    pub(crate) size: usize,
    pub(crate) sorts: Vec<Sort>,
    pub(crate) fields: Vec<String>,
    pub filter: T,
    pub(crate) pageable: bool,
}

impl<T> Condition<T>
where
    T: Filter,
{
    pub fn new(t: T) -> Self {
        Self {
            db: Default::default(),
            table: Default::default(),
            pageable: false,
            page: 0,
            size: 10,
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

    pub fn with_sort(&mut self, sorts: Vec<Sort>) -> &mut Condition<T> {
        self.sorts = sorts;
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

    pub fn with_page(&mut self, page: usize, size: usize) -> &mut Condition<T> {
        self.page = page;
        self.size = size;
        self.pageable = true;
        self
    }

    pub fn wheres<S: ToString + ?Sized>(&mut self, input: &S) -> anyhow::Result<&mut Self> {
        self.filter.parse(&input.to_string())?;
        Ok(self)
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
