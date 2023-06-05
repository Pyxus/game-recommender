pub fn comma_sep<T, F, C>(collection: C, f: F) -> String
where
    F: FnMut(&T) -> String,
    C: IntoIterator<Item = T>,
{
    let coll_vec: Vec<T> = collection.into_iter().collect();
    coll_vec.iter().map(f).collect::<Vec<String>>().join(", ")
}