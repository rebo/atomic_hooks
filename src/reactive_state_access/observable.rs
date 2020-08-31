pub trait Observable<T>
where
    T: 'static,
{
    // fn id(&self) -> StorageKey;
    fn observe(&self) -> T
    where
        T: Clone + 'static;
    fn observe_update(&self) -> (Option<T>, T)
    where
        T: Clone + 'static;
    fn observe_with<F: FnOnce(&T) -> R, R>(&self, func: F) -> R;
}
