use anyhow::Result;

pub fn flatten<T>(left: Result<Vec<T>>, right: T) -> Result<Vec<T>> {
    let mut flt = left?;
    flt.push(right);
    Ok(flt)
}
