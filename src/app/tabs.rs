pub mod budget;
pub mod budget_summary;
pub mod statement;

#[derive(PartialEq)]
pub enum Tabs {
    Statement,
    BudgetSummary,
    Budget,
}
