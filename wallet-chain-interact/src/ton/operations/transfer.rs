pub struct TransferOpt {
    pub from: String,
    pub to: String,
    pub value: String,
}

impl TransferOpt {
    pub fn new(from: &str, to: &str, value: &str) -> crate::Result<Self> {
        Ok(Self {
            from: from.to_string(),
            to: to.to_string(),
            value: value.to_string(),
        })
    }
}
