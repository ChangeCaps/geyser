#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sharing<T> {
    Exclusive,
    Concurrent(T),
}

