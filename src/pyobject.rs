pub enum PyObject {
    Int(i32, bool),
    Str(&'static str, bool),
    Ascii(&'static str, bool),
    None(bool)
}
