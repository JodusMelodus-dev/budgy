pub mod statement;
pub mod budget_summary;
pub mod budget;

#[derive(PartialEq)]
pub enum Tabs {
    Statement,
    BudgetSummary,
    Budget,
}
