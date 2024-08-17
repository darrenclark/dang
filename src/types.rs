#[derive(
    Debug,
    Clone,
    Copy,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    Hash,
    strum::Display,
    strum::AsRefStr,
    strum::EnumIs,
)]
#[strum(serialize_all = "snake_case")]
pub enum Type {
    Nil,
    Symbol,
    Bool,
    String,
    Int,
    Function,
    Tuple,
    List,
    Dict,
    Struct,
}
