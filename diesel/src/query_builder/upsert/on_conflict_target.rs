use crate::backend::sql_dialect;
use crate::expression::SqlLiteral;
use crate::query_builder::*;
use crate::query_source::Column;

#[doc(hidden)]
pub trait OnConflictTarget<Table> {}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, QueryId)]
pub struct NoConflictTarget;

impl<DB> QueryFragment<DB> for NoConflictTarget
where
    DB: Backend,
    DB::OnConflictClause: sql_dialect::on_conflict_clause::SupportsOnConflictClause,
{
    fn walk_ast<'b>(&'b self, _: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        Ok(())
    }
}

impl<Table> OnConflictTarget<Table> for NoConflictTarget {}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, QueryId)]
pub struct ConflictTarget<T>(pub T);

impl<DB, T> QueryFragment<DB> for ConflictTarget<T>
where
    DB: Backend,
    Self: QueryFragment<DB, DB::OnConflictClause>,
{
    fn walk_ast<'b>(&'b self, pass: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        <Self as QueryFragment<DB, DB::OnConflictClause>>::walk_ast(self, pass)
    }
}

impl<DB, T, SP> QueryFragment<DB, SP> for ConflictTarget<T>
where
    DB: Backend<OnConflictClause = SP>,
    SP: sql_dialect::on_conflict_clause::PgLikeOnConflictClause,
    T: Column,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.push_sql(" (");
        out.push_identifier(T::NAME)?;
        out.push_sql(")");
        Ok(())
    }
}

impl<T> OnConflictTarget<T::Table> for ConflictTarget<T> where T: Column {}

impl<DB, ST, SP> QueryFragment<DB, SP> for ConflictTarget<SqlLiteral<ST>>
where
    DB: Backend<OnConflictClause = SP>,
    SP: sql_dialect::on_conflict_clause::PgLikeOnConflictClause,
    SqlLiteral<ST>: QueryFragment<DB>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, DB>) -> QueryResult<()> {
        out.push_sql(" ");
        self.0.walk_ast(out.reborrow())?;
        Ok(())
    }
}

impl<Tab, ST> OnConflictTarget<Tab> for ConflictTarget<SqlLiteral<ST>> {}

#[diesel_derives::expand_for_tuple(1..)]
impl<_DB, _SP, T: Tuple, Table> QueryFragment<_DB, _SP> for ConflictTarget<T>
where
    _DB: Backend<OnConflictClause = _SP>,
    _SP: sql_dialect::on_conflict_clause::PgLikeOnConflictClause,
    T<_>: Column<Table = Table>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, _DB>) -> QueryResult<()> {
        out.push_sql(" (");
        out.push_identifier(T::<0>::NAME)?;
        for typle_index!(i) in 1..T::LEN {
            out.push_sql(", ");
            out.push_identifier(T::<{ i }>::NAME)?;
        }
        out.push_sql(")");
        Ok(())
    }
}

#[diesel_derives::expand_for_tuple(1..)]
impl<T: Tuple, Table> OnConflictTarget<Table> for ConflictTarget<T> where T<_>: Column<Table = Table>
{}
