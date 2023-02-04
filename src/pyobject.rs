pub enum PyObject <'a> {
    Int(i32, bool),
    Float(f64, bool),
    Str(&'a str, bool),
    Ascii(&'a str, bool),
    None(bool),
    True(bool),
    False(bool),
}
