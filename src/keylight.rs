#[derive(Clone)]
pub struct Keylight {
    pub url: String,
    pub on: bool,
}

impl Keylight {
    pub fn new(url: &str, on: bool) -> Self {
        Self {
            url: url.to_string(),
            on: on,
        }
    }
}
