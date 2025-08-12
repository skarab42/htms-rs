use crate::task::Task;

pub trait Render: Sized {
    fn template() -> String;
    fn tasks() -> Option<Vec<Task>>;

    #[must_use]
    fn render() -> String {
        Self::template()
    }
}
