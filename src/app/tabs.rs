pub mod budget;
pub mod budget_summary;
pub mod statement;

#[derive(PartialEq, Eq)]
pub enum Tabs {
    Statement,
    BudgetSummary,
    Budget,
}
