pub mod budget;
pub mod budget_summary;
pub mod dashboard;
pub mod settings;
pub mod statement;

#[derive(PartialEq, Eq)]
pub enum Tabs {
    Dashboard,
    Statement,
    BudgetSummary,
    Budget,
    Settings,
}
