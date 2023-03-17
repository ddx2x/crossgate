use condition::Validate as Expr;
use serde_json::Value;

use super::Unstructed;

pub fn validate_match<'a>(
    src: Option<&'a mut Unstructed>,
    tag: &'a mut Unstructed,
    expr: Expr,
) -> bool {
    match expr {
        Expr::And { span, lhs, rhs } => todo!(),
        Expr::Or { span, lhs, rhs } => todo!(),
        Expr::Eq { span, field, value } => todo!(),
        Expr::Ne { span, field, value } => todo!(),
        Expr::Gt { span, field, value } => todo!(),
        Expr::Gte { span, field, value } => todo!(),
        Expr::Lt { span, field, value } => todo!(),
        Expr::Lte { span, field, value } => todo!(),
        Expr::In { span, field, value } => todo!(),
        Expr::NotIn { span, field, value } => todo!(),
        Expr::IsNull { span, field, value } => todo!(),
        Expr::IsNotNull { span, field, value } => todo!(),
        Expr::IsNumber { span, field, value } => todo!(),
        Expr::IsString { span, field, value } => todo!(),
        Expr::LenField { span, field, expr, value } => todo!(),
        Expr::Join { from, expr, field, value } => todo!(),
    }
    false
}
