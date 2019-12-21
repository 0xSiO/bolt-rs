use failure::Fail;

#[derive(Debug, Fail)]
enum ValueError {
    #[fail(display = "Value too large: {}", size)]
    ValueTooLarge { size: usize },
}
