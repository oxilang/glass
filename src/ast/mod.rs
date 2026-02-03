use thin_vec::ThinVec;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum Value {
    Map(ThinVec<(Box<str>, Value)>),
    Array(ThinVec<Value>),
    String(String),
    Number(f64),
    Bool(bool),
}
