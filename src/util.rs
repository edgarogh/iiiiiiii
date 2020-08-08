pub trait InnerMatches<T> {
    fn inner_is<F: FnOnce(T) -> bool>(self, predicate: F) -> bool;
}

impl<T> InnerMatches<T> for Option<T> {
    fn inner_is<F: FnOnce(T) -> bool>(self, predicate: F) -> bool {
        match self {
            Some(value) => predicate(value),
            None => false,
        }
    }
}

impl<T, E> InnerMatches<T> for Result<T, E> {
    fn inner_is<F: FnOnce(T) -> bool>(self, predicate: F) -> bool {
        match self {
            Ok(value) => predicate(value),
            Err(_) => false,
        }
    }
}
