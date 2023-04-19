use std::error::Error;

pub fn flatten<T>(
    left: Result<Vec<T>, Box<dyn Error>>,
    right: T,
) -> Result<Vec<T>, Box<dyn Error>> {
    let mut flt = left?;
    flt.push(right);
    Ok(flt)
}

pub fn gen_error(func: &str, rule: &str) -> Box<dyn Error> {
    format!("Parse Error in {}: {}", func, rule).into()
}
