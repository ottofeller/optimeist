#[derive(Debug, Default)]
pub enum LoadState<T> {
    #[default]
    Loading,
    Loaded(T),
    Failed(color_eyre::Report),
}
