use crate::associations::BelongsTo;
use crate::backend::Backend;
use crate::deserialize::{
    self, FromSqlRow, FromStaticSqlRow, Queryable, SqlTypeOrSelectable, StaticallySizedRow,
};
use crate::expression::{
    is_contained_in_group_by, AppearsOnTable, Expression, IsContainedInGroupBy, MixedAggregates,
    QueryMetadata, Selectable, SelectableExpression, TypedExpressionType, ValidGrouping,
};
use crate::insertable::{CanInsertInSingleQuery, InsertValues, Insertable, InsertableOptionHelper};
use crate::query_builder::*;
use crate::query_dsl::load_dsl::CompatibleType;
use crate::query_source::*;
use crate::result::QueryResult;
use crate::row::*;
use crate::sql_types::{HasSqlType, IntoNullable, Nullable, OneIsNullable, SqlType};
use crate::util::{TupleAppend, TupleSize};

impl<T> TupleSize for T
where
    T: crate::sql_types::SingleValue,
{
    const SIZE: usize = 1;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> HasSqlType<T> for __DB
where
    typle!(i in .. => __DB: HasSqlType<T<{i}>>): Tuple::Bounds,
    __DB: Backend,
{
    fn metadata(_: &mut __DB::MetadataLookup) -> __DB::TypeMetadata {
        unreachable!("Tuples should never implement `ToSql` directly");
    }
}

#[diesel_derives::expand_for_tuple(1..)]
#[diagnostic::do_not_recommend]
impl<T: Tuple, U, ST: Tuple, __DB> FromSqlRow<(ST<{ .. }>, crate::sql_types::Untyped), __DB>
    for (T<{ .. }>, U)
where
    __DB: Backend,
    U: FromSqlRow<crate::sql_types::Untyped, __DB>,
    typle!(i in .. => T<{i}>: FromSqlRow<ST<{i}>, __DB>): Tuple::Bounds,
    typle!(i in .. => T<{i}>: StaticallySizedRow<ST<{i}>, __DB>): Tuple::Bounds,
{
    fn build_from_row<'a>(full_row: &impl Row<'a, __DB>) -> deserialize::Result<Self> {
        let field_count = full_row.field_count();
        let mut static_field_count = 0;

        Ok((
            typle!(i in .. => {
                let row = full_row
                    .partial_row(static_field_count..static_field_count + T::<{i}>::FIELD_COUNT);
                static_field_count += T::<{i}>::FIELD_COUNT;
                T::<{i}>::build_from_row(&row)?
            }),
            U::build_from_row(&full_row.partial_row(static_field_count..field_count))?,
        ))
    }
}

#[diesel_derives::expand_for_tuple(1..)]
#[diagnostic::do_not_recommend]
impl<T: Tuple> Expression for T
where
    T<_>: Expression,
    (typle! {i in .. => T<{i}>::SqlType}): TypedExpressionType,
{
    type SqlType = (typle! {i in .. => <T<{i}> as Expression>::SqlType});
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> TypedExpressionType for T where T<_>: TypedExpressionType {}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> TypedExpressionType for Nullable<T>
where
    T<_>: SqlType + TypedExpressionType,
    T: SqlType,
{
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> IntoNullable for T
where
    T<_>: SqlType,
    Self: SqlType,
{
    type Nullable = Nullable<T>;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> Selectable<__DB> for T
where
    __DB: Backend,
    T<_>: Selectable<__DB>,
{
    type SelectExpression = (typle! {i in .. => T<{i}>::SelectExpression});

    fn construct_selection() -> Self::SelectExpression {
        (typle! {i in .. => T::<{i}>::construct_selection()})
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB: Backend> QueryFragment<__DB> for T
where
    T<_>: QueryFragment<__DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, __DB>) -> QueryResult<()> {
        #[typle_attr_if(T::LEN == 1, allow(unused_mut))]
        let mut needs_comma = false;
        for typle_index!(i) in 0..T::LEN {
            if !self[[i]].is_noop(out.backend())? {
                if needs_comma {
                    out.push_sql(", ");
                }
                self[[i]].walk_ast(out.reborrow())?;
                if typle_const!(i < T::LAST) {
                    needs_comma = true;
                }
            }
        }
        Ok(())
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Tab> ColumnList for T
where
    T<_>: ColumnList<Table = Tab>,
{
    type Table = Tab;

    fn walk_ast<__DB: Backend>(&self, mut out: AstPass<'_, '_, __DB>) -> QueryResult<()> {
        self.0.walk_ast(out.reborrow())?;
        for typle_index!(i) in 1..T::LEN {
            out.push_sql(", ");
            self[[i]].walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> QueryId for T
where
    T<_>: QueryId,
{
    type QueryId = (typle!(i in .. => T<{i}>::QueryId));

    const HAS_STATIC_QUERY_ID: bool = typle_all!(
        i in .. => T::<{i}>::HAS_STATIC_QUERY_ID
    );
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __GroupByClause> ValidGrouping<__GroupByClause> for T
where
    T<0>: ValidGrouping<__GroupByClause>,
    (T<{ 1.. }>): ValidGrouping<__GroupByClause>,
    T<0>::IsAggregate:
        MixedAggregates<<(T<{ 1.. }>) as ValidGrouping<__GroupByClause>>::IsAggregate>,
{
    type IsAggregate = <T<0>::IsAggregate as MixedAggregates<
        <(T<{ 1.. }>) as ValidGrouping<__GroupByClause>>::IsAggregate,
    >>::Output;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Col> IsContainedInGroupBy<Col> for T
where
    Col: QueryRelationField,
    (T<{ 1.. }>): IsContainedInGroupBy<Col>,
    T<0>: IsContainedInGroupBy<Col>,
    T<0>::Output:
        is_contained_in_group_by::IsAny<<(T<{ 1.. }>) as IsContainedInGroupBy<Col>>::Output>,
{
    type Output = <T<{ 0 }>::Output as is_contained_in_group_by::IsAny<
        <(T<{ 1.. }>) as IsContainedInGroupBy<Col>>::Output,
    >>::Output;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Tab> UndecoratedInsertRecord<Tab> for T where T<_>: UndecoratedInsertRecord<Tab> {}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> CanInsertInSingleQuery<__DB> for T
where
    __DB: Backend,
    T<_>: CanInsertInSingleQuery<__DB>,
{
    fn rows_to_insert(&self) -> Option<usize> {
        for typle_index!(i) in 0..T::LEN {
            debug_assert_eq!(typle_expr!(self[[i]]).rows_to_insert(), Some(1));
        }
        Some(1)
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, ST: Tuple, Tab> Insertable<Tab> for T
where
    typle!(i in .. => T<{i}>: Insertable<Tab, Values = ValuesClause<ST<{i}>, Tab>>): Tuple::Bounds,
{
    type Values = ValuesClause<ST, Tab>;

    fn values(self) -> Self::Values {
        ValuesClause::new((typle!(i in .. => self[[i]].values().values)))
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<'a, T: Tuple, Tab> Insertable<Tab> for &'a T
where
    (typle!(i in .. => &'a T<{i}>)): Insertable<Tab>,
{
    type Values = <(typle!(i in .. => &'a T<{i}>)) as Insertable<Tab>>::Values;

    fn values(self) -> Self::Values {
        (typle!(i in .. => &self[[i]])).values()
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Tab, __DB> InsertValues<__DB, Tab> for T
where
    Tab: Table,
    __DB: Backend,
    T<_>: InsertValues<__DB, Tab>,
{
    fn column_names(&self, mut out: AstPass<'_, '_, __DB>) -> QueryResult<()> {
        #[typle_attr_if(T::LEN == 1, allow(unused_mut))]
        let mut needs_comma = false;
        for typle_index!(i) in 0..T::LEN {
            let noop_element = self[[i]].is_noop(out.backend())?;
            if !noop_element {
                if needs_comma {
                    out.push_sql(", ");
                }
                self[[i]].column_names(out.reborrow())?;
                if typle_const!(i < T::LAST) {
                    needs_comma = true;
                }
            }
        }
        Ok(())
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<__T, ST: Tuple, Tab> Insertable<Tab> for InsertableOptionHelper<__T, ST>
where
    __T: Insertable<Tab>,
    __T::Values: Default,
{
    type Values = __T::Values;

    fn values(self) -> Self::Values {
        self.0.map(|v| v.values()).unwrap_or_default()
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, QS> SelectableExpression<QS> for T
where
    T<_>: SelectableExpression<QS>,
    T: AppearsOnTable<QS>,
{
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, QS> AppearsOnTable<QS> for T
where
    T<_>: AppearsOnTable<QS>,
    T: Expression,
{
}

#[diesel_derives::expand_for_tuple(1..)]
impl<Target, T: Tuple> AsChangeset for T
where
    T<_>: AsChangeset<Target = Target>,
    Target: QuerySource,
{
    type Target = Target;

    type Changeset = (typle!(i in .. => T<{i}>::Changeset));

    fn as_changeset(self) -> Self::Changeset {
        (typle!(i in .. => self[[i]].as_changeset()))
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Parent> BelongsTo<Parent> for T
where
    T<0>: BelongsTo<Parent>,
{
    type ForeignKey = T<0>::ForeignKey;
    type ForeignKeyColumn = T<0>::ForeignKeyColumn;
    fn foreign_key(&self) -> Option<&Self::ForeignKey> {
        self.0.foreign_key()
    }
    fn foreign_key_column() -> Self::ForeignKeyColumn {
        T::<0>::foreign_key_column()
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Next> TupleAppend<Next> for T {
    type Output = (T<{ .. }>, Next);

    fn tuple_append(self, next: Next) -> Self::Output {
        (self[[..]], next)
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> SqlType for T
where
    T<_>: SqlType,
    typle!(
        i in 1.. => T<{i}>::IsNull:
        OneIsNullable<typle_fold!(T<0>::IsNull;
            j in 1..i => |X| <T<{j}>::IsNull as OneIsNullable<X>>::Out
        )>
    ): Tuple::Bounds,
{
    type IsNull = typle_fold!(
        T::<{0}>::IsNull;
        i in 1.. => |X| <T::<{i}>::IsNull as OneIsNullable<X>>::Out
    );
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB, ST: Tuple> Queryable<ST, __DB> for T
where
    __DB: Backend,
    Self: FromStaticSqlRow<ST, __DB>,
{
    type Row = Self;

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(row)
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<__T, ST: Tuple, __DB> FromStaticSqlRow<Nullable<ST>, __DB> for Option<__T>
where
    __DB: Backend,
    ST: SqlType,
    __T: FromSqlRow<ST, __DB>,
{
    fn build_from_row<'a>(row: &impl Row<'a, __DB>) -> deserialize::Result<Self> {
        match <__T as FromSqlRow<ST, __DB>>::build_from_row(row) {
            Ok(v) => Ok(Some(v)),
            Err(e) if e.is::<crate::result::UnexpectedNullError>() => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<__T, __DB, ST: Tuple> Queryable<Nullable<ST>, __DB> for Option<__T>
where
    __DB: Backend,
    Self: FromStaticSqlRow<Nullable<ST>, __DB>,
    ST: SqlType,
{
    type Row = Self;

    fn build(row: Self::Row) -> deserialize::Result<Self> {
        Ok(row)
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> TupleSize for T
where
    T<_>: TupleSize,
{
    const SIZE: usize = typle_fold!(0; i in .. => |size| size + T::<{i}>::SIZE);
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple> TupleSize for Nullable<T>
where
    T<_>: TupleSize,
    T: SqlType,
{
    const SIZE: usize = typle_fold!(0; i in .. => |size| size + T::<{i}>::SIZE);
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> QueryMetadata<T> for __DB
where
    __DB: Backend,
    typle!(i in .. => __DB: QueryMetadata<T<{i}>>): Tuple::Bounds,
{
    fn row_metadata(lookup: &mut Self::MetadataLookup, row: &mut Vec<Option<__DB::TypeMetadata>>) {
        for typle_index!(i) in 0..T::LEN {
            <__DB as QueryMetadata<T<{ i }>>>::row_metadata(lookup, row);
        }
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> QueryMetadata<Nullable<T>> for __DB
where
    __DB: Backend,
    typle!(i in .. => __DB: QueryMetadata<T<{i}>>): Tuple::Bounds,
{
    fn row_metadata(lookup: &mut Self::MetadataLookup, row: &mut Vec<Option<__DB::TypeMetadata>>) {
        for typle_index!(i) in 0..T::LEN {
            <__DB as QueryMetadata<T<{ i }>>>::row_metadata(lookup, row);
        }
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, __DB> deserialize::QueryableByName<__DB> for T
where
    __DB: Backend,
    T<_>: deserialize::QueryableByName<__DB>,
{
    fn build<'a>(row: &impl NamedRow<'a, __DB>) -> deserialize::Result<Self> {
        Ok((typle!(i in .. => <T<{i}> as deserialize::QueryableByName<__DB>>::build(row)?)))
    }
}

#[diesel_derives::expand_for_tuple(1..)]
#[diagnostic::do_not_recommend]
impl<__T, ST: Tuple, __DB> CompatibleType<__T, __DB> for ST
where
    __DB: Backend,
    __T: FromSqlRow<ST, __DB>,
{
    type SqlType = Self;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<__T, ST: Tuple, __DB> CompatibleType<Option<__T>, __DB> for Nullable<ST>
where
    __DB: Backend,
    ST: CompatibleType<__T, __DB>,
{
    type SqlType = Nullable<<ST as CompatibleType<__T, __DB>>::SqlType>;
}

#[diesel_derives::expand_for_tuple(1..)]
impl<ST: Tuple> SqlTypeOrSelectable for ST where ST<_>: SqlTypeOrSelectable {}

#[diesel_derives::expand_for_tuple(1..)]
impl<ST: Tuple> SqlTypeOrSelectable for Nullable<ST> where ST: SqlTypeOrSelectable {}

impl<T0, ST0, __DB> FromStaticSqlRow<(ST0,), __DB> for (T0,)
where
    __DB: Backend,
    ST0: CompatibleType<T0, __DB>,
    T0: FromSqlRow<<ST0 as CompatibleType<T0, __DB>>::SqlType, __DB>,
{
    fn build_from_row<'a>(row: &impl Row<'a, __DB>) -> deserialize::Result<Self> {
        Ok((T0::build_from_row(row)?,))
    }
}

#[diesel_derives::expand_for_tuple(2..)]
impl<T: Tuple, ST: Tuple, __DB> FromStaticSqlRow<ST, __DB> for T
where
    __DB: Backend,
    typle!(i in .. => ST<{i}>: CompatibleType<T<{i}>, __DB>): Tuple::Bounds,
    typle!(i in .. => T<{i}>: FromSqlRow<<ST<{i}> as CompatibleType<T<{i}>, __DB>>::SqlType, __DB>):
        Tuple::Bounds,
    typle!(i in ..T::LAST => T<{i}>: StaticallySizedRow<<ST<{i}> as CompatibleType<T<{i}>, __DB>>::SqlType, __DB>):
        Tuple::Bounds,
{
    fn build_from_row<'a>(full_row: &impl Row<'a, __DB>) -> deserialize::Result<Self> {
        let field_count = full_row.field_count();
        let mut static_field_count = 0;

        Ok((
            typle!(i in 0..T::LAST => {
                let row = full_row
                    .partial_row(static_field_count..static_field_count + T::<{i}>::FIELD_COUNT);
                static_field_count += T::<{i}>::FIELD_COUNT;
                <T<{i}> as FromSqlRow<
                    <ST<{i}> as CompatibleType<T<{i}>, __DB>>::SqlType,
                    __DB,
                >>::build_from_row(&row)?
            }),
            T::<{ T::LAST }>::build_from_row(
                &full_row.partial_row(static_field_count..field_count),
            )?,
        ))
    }
}
