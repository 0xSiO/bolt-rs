use failure::Fail;

#[derive(Debug, Fail)]
pub enum ValueError {
    #[fail(display = "Value too large (length {})", _0)]
    TooLarge(usize),
}

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct SerializeError(pub String);

#[derive(Debug, Fail)]
#[fail(display = "{}", _0)]
pub struct DeserializeError(pub String);
