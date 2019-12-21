use failure::Fail;

#[derive(Debug, Fail)]
pub enum ValueError {
    #[fail(display = "Value too large: {}", size)]
    ValueTooLarge { size: usize },
}
