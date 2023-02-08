pub enum PyObject <'a> {
    Int(i32, bool),
    Float(f64, bool),
    String(&'a str, bool), // 文字列ではなくバイト列に利用
    Ascii(&'a str, bool),
    AsciiShort(&'a str, bool),
    Unicode(&'a str, bool),
    None(bool),
    True(bool),
    False(bool),
}
