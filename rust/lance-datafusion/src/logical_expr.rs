// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright The Lance Authors

//! Extends logical expression.

use std::sync::Arc;

use arrow_schema::DataType;

use crate::expr::safe_coerce_scalar;
use datafusion::logical_expr::{expr::ScalarFunction, BinaryExpr, Operator};
use datafusion::logical_expr::{Between, ScalarUDF, ScalarUDFImpl};
use datafusion::prelude::*;
use datafusion::scalar::ScalarValue;
use datafusion_functions::core::getfield::GetFieldFunc;
use lance_arrow::DataTypeExt;

use lance_core::datatypes::Schema;
use lance_core::{Error, Result};
use snafu::location;
/// Resolve a Value
fn resolve_value(expr: &Expr, data_type: &DataType) -> Result<Expr> {
    match expr {
        Expr::Literal(scalar_value, metadata) => {
            Ok(Expr::Literal(safe_coerce_scalar(scalar_value, data_type).ok_or_else(|| Error::invalid_input(
                format!("Received literal {expr} and could not convert to literal of type '{data_type:?}'"),
                location!(),
            ))?, metadata.clone()))
        }
        _ => Err(Error::invalid_input(
            format!("Expected a literal of type '{data_type:?}' but received: {expr}"),
            location!(),
        )),
    }
}

/// A simple helper function that interprets an Expr as a string scalar
/// or returns None if it is not.
pub fn get_as_string_scalar_opt(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Literal(ScalarValue::Utf8(Some(s)), _) => Some(s),
        _ => None,
    }
}

/// Given a Expr::Column or Expr::GetIndexedField, get the data type of referenced
/// field in the schema.
///
/// If the column is not found in the schema, return None. If the expression is
/// not a field reference, also returns None.
pub fn resolve_column_type(expr: &Expr, schema: &Schema) -> Option<DataType> {
    let mut field_path = Vec::new();
    let mut current_expr = expr;
    // We are looping from outer-most reference to inner-most.
    loop {
        match current_expr {
            Expr::Column(c) => {
                field_path.push(c.name.as_str());
                break;
            }
            Expr::ScalarFunction(udf) => {
                if udf.name() == GetFieldFunc::default().name() {
                    let name = get_as_string_scalar_opt(&udf.args[1])?;
                    field_path.push(name);
                    current_expr = &udf.args[0];
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }

    let mut path_iter = field_path.iter().rev();
    let mut field = schema.field(path_iter.next()?)?;
    for name in path_iter {
        if field.data_type().is_struct() {
            field = field.children.iter().find(|f| &f.name == name)?;
        } else {
            return None;
        }
    }
    Some(field.data_type())
}

/// Resolve logical expression `expr`.
///
/// Parameters
///
/// - *expr*: a datafusion logical expression
/// - *schema*: lance schema.
pub fn resolve_expr(expr: &Expr, schema: &Schema) -> Result<Expr> {
    match expr {
        Expr::Between(Between {
            expr: inner_expr,
            low,
            high,
            negated,
        }) => {
            if let Some(inner_expr_type) = resolve_column_type(inner_expr.as_ref(), schema) {
                Ok(Expr::Between(Between {
                    expr: inner_expr.clone(),
                    low: Box::new(coerce_expr(low.as_ref(), &inner_expr_type)?),
                    high: Box::new(coerce_expr(high.as_ref(), &inner_expr_type)?),
                    negated: *negated,
                }))
            } else {
                Ok(expr.clone())
            }
        }
        Expr::BinaryExpr(BinaryExpr { left, op, right }) => {
            if matches!(op, Operator::And | Operator::Or) {
                Ok(Expr::BinaryExpr(BinaryExpr {
                    left: Box::new(resolve_expr(left.as_ref(), schema)?),
                    op: *op,
                    right: Box::new(resolve_expr(right.as_ref(), schema)?),
                }))
            } else if let Some(left_type) = resolve_column_type(left.as_ref(), schema) {
                match right.as_ref() {
                    Expr::Literal(..) => Ok(Expr::BinaryExpr(BinaryExpr {
                        left: left.clone(),
                        op: *op,
                        right: Box::new(resolve_value(right.as_ref(), &left_type)?),
                    })),
                    // For cases complex expressions (not just literals) on right hand side like x = 1 + 1 + -2*2
                    Expr::BinaryExpr(r) => Ok(Expr::BinaryExpr(BinaryExpr {
                        left: left.clone(),
                        op: *op,
                        right: Box::new(Expr::BinaryExpr(BinaryExpr {
                            left: coerce_expr(&r.left, &left_type).map(Box::new)?,
                            op: r.op,
                            right: coerce_expr(&r.right, &left_type).map(Box::new)?,
                        })),
                    })),
                    _ => Ok(expr.clone()),
                }
            } else if let Some(right_type) = resolve_column_type(right.as_ref(), schema) {
                match left.as_ref() {
                    Expr::Literal(..) => Ok(Expr::BinaryExpr(BinaryExpr {
                        left: Box::new(resolve_value(left.as_ref(), &right_type)?),
                        op: *op,
                        right: right.clone(),
                    })),
                    _ => Ok(expr.clone()),
                }
            } else {
                Ok(expr.clone())
            }
        }
        Expr::InList(in_list) => {
            if matches!(in_list.expr.as_ref(), Expr::Column(_)) {
                if let Some(resolved_type) = resolve_column_type(in_list.expr.as_ref(), schema) {
                    let resolved_values = in_list
                        .list
                        .iter()
                        .map(|val| coerce_expr(val, &resolved_type))
                        .collect::<Result<Vec<_>>>()?;
                    Ok(Expr::in_list(
                        in_list.expr.as_ref().clone(),
                        resolved_values,
                        in_list.negated,
                    ))
                } else {
                    Ok(expr.clone())
                }
            } else {
                Ok(expr.clone())
            }
        }
        _ => {
            // Passthrough
            Ok(expr.clone())
        }
    }
}

/// Coerce expression of literals to column type.
///
/// Parameters
///
/// - *expr*: a datafusion logical expression
/// - *dtype*: a lance data type
pub fn coerce_expr(expr: &Expr, dtype: &DataType) -> Result<Expr> {
    match expr {
        Expr::BinaryExpr(BinaryExpr { left, op, right }) => Ok(Expr::BinaryExpr(BinaryExpr {
            left: Box::new(coerce_expr(left, dtype)?),
            op: *op,
            right: Box::new(coerce_expr(right, dtype)?),
        })),
        literal_expr @ Expr::Literal(..) => Ok(resolve_value(literal_expr, dtype)?),
        _ => Ok(expr.clone()),
    }
}

/// Coerce logical expression for filters to boolean.
///
/// Parameters
///
/// - *expr*: a datafusion logical expression
pub fn coerce_filter_type_to_boolean(expr: Expr) -> Expr {
    match &expr {
        // TODO: consider making this dispatch more generic, i.e. fun.output_type -> coerce
        // instead of hardcoding coerce method for each function
        Expr::ScalarFunction(ScalarFunction { func, .. }) => {
            if func.name() == "regexp_match" {
                Expr::IsNotNull(Box::new(expr))
            } else {
                expr
            }
        }
        _ => expr,
    }
}

// As part of the DF 37 release there are now two different ways to
// represent a nested field access in `Expr`.  The old way is to use
// `Expr::field` which returns a `GetStructField` and the new way is
// to use `Expr::ScalarFunction` with a `GetFieldFunc` UDF.
//
// Currently, the old path leads to bugs in DF.  This is probably a
// bug and will probably be fixed in a future version.  In the meantime
// we need to make sure we are always using the new way to avoid this
// bug.  This trait adds field_newstyle which lets us easily create
// logical `Expr` that use the new style.
pub trait ExprExt {
    // Helper function to replace Expr::field in DF 37 since DF
    // confuses itself with the GetStructField returned by Expr::field
    fn field_newstyle(&self, name: &str) -> Expr;
}

impl ExprExt for Expr {
    fn field_newstyle(&self, name: &str) -> Expr {
        Self::ScalarFunction(ScalarFunction {
            func: Arc::new(ScalarUDF::new_from_impl(GetFieldFunc::default())),
            args: vec![
                self.clone(),
                Self::Literal(ScalarValue::Utf8(Some(name.to_string())), None),
            ],
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use super::*;

    use arrow_schema::{Field, Schema as ArrowSchema};
    use datafusion_functions::core::expr_ext::FieldAccessor;

    #[test]
    fn test_resolve_large_utf8() {
        let arrow_schema = ArrowSchema::new(vec![Field::new("a", DataType::LargeUtf8, false)]);
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(Expr::Column("a".to_string().into())),
            op: Operator::Eq,
            right: Box::new(Expr::Literal(
                ScalarValue::Utf8(Some("a".to_string())),
                None,
            )),
        });

        let resolved = resolve_expr(&expr, &Schema::try_from(&arrow_schema).unwrap()).unwrap();
        match resolved {
            Expr::BinaryExpr(be) => {
                assert_eq!(
                    be.right.as_ref(),
                    &Expr::Literal(ScalarValue::LargeUtf8(Some("a".to_string())), None)
                )
            }
            _ => unreachable!("Expected BinaryExpr"),
        };
    }

    #[test]
    fn test_resolve_binary_expr_on_right() {
        let arrow_schema = ArrowSchema::new(vec![Field::new("a", DataType::Float64, false)]);
        let expr = Expr::BinaryExpr(BinaryExpr {
            left: Box::new(Expr::Column("a".to_string().into())),
            op: Operator::Eq,
            right: Box::new(Expr::BinaryExpr(BinaryExpr {
                left: Box::new(Expr::Literal(ScalarValue::Int64(Some(2)), None)),
                op: Operator::Minus,
                right: Box::new(Expr::Literal(ScalarValue::Int64(Some(-1)), None)),
            })),
        });
        let resolved = resolve_expr(&expr, &Schema::try_from(&arrow_schema).unwrap()).unwrap();

        match resolved {
            Expr::BinaryExpr(be) => match be.right.as_ref() {
                Expr::BinaryExpr(r_be) => {
                    assert_eq!(
                        r_be.left.as_ref(),
                        &Expr::Literal(ScalarValue::Float64(Some(2.0)), None)
                    );
                    assert_eq!(
                        r_be.right.as_ref(),
                        &Expr::Literal(ScalarValue::Float64(Some(-1.0)), None)
                    );
                }
                _ => panic!("Expected BinaryExpr"),
            },
            _ => panic!("Expected BinaryExpr"),
        }
    }

    #[test]
    fn test_resolve_in_expr() {
        // Type coercion should apply for `A IN (0)` or `A NOT IN (0)`
        let arrow_schema = ArrowSchema::new(vec![Field::new("a", DataType::Float32, false)]);
        let expr = Expr::in_list(
            Expr::Column("a".to_string().into()),
            vec![Expr::Literal(ScalarValue::Float64(Some(0.0)), None)],
            false,
        );
        let resolved = resolve_expr(&expr, &Schema::try_from(&arrow_schema).unwrap()).unwrap();
        let expected = Expr::in_list(
            Expr::Column("a".to_string().into()),
            vec![Expr::Literal(ScalarValue::Float32(Some(0.0)), None)],
            false,
        );
        assert_eq!(resolved, expected);

        let expr = Expr::in_list(
            Expr::Column("a".to_string().into()),
            vec![Expr::Literal(ScalarValue::Float64(Some(0.0)), None)],
            true,
        );
        let resolved = resolve_expr(&expr, &Schema::try_from(&arrow_schema).unwrap()).unwrap();
        let expected = Expr::in_list(
            Expr::Column("a".to_string().into()),
            vec![Expr::Literal(ScalarValue::Float32(Some(0.0)), None)],
            true,
        );
        assert_eq!(resolved, expected);
    }

    #[test]
    fn test_resolve_column_type() {
        let schema = Arc::new(ArrowSchema::new(vec![
            Field::new("int", DataType::Int32, true),
            Field::new(
                "st",
                DataType::Struct(
                    vec![
                        Field::new("str", DataType::Utf8, true),
                        Field::new(
                            "st",
                            DataType::Struct(
                                vec![Field::new("float", DataType::Float64, true)].into(),
                            ),
                            true,
                        ),
                    ]
                    .into(),
                ),
                true,
            ),
        ]));
        let schema = Schema::try_from(schema.as_ref()).unwrap();

        assert_eq!(
            resolve_column_type(&col("int"), &schema),
            Some(DataType::Int32)
        );
        assert_eq!(
            resolve_column_type(&col("st").field("str"), &schema),
            Some(DataType::Utf8)
        );
        assert_eq!(
            resolve_column_type(&col("st").field("st").field("float"), &schema),
            Some(DataType::Float64)
        );

        assert_eq!(resolve_column_type(&col("x"), &schema), None);
        assert_eq!(resolve_column_type(&col("str"), &schema), None);
        assert_eq!(resolve_column_type(&col("float"), &schema), None);
        assert_eq!(
            resolve_column_type(&col("st").field("str").eq(lit("x")), &schema),
            None
        );
    }
}
