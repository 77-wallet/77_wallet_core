#[derive(Debug)]
pub enum Language {
    English,
    ChineseSimplified,
    ChineseTraditional,
    Czech,
    French,
    Italian,
    Japanese,
    Korean,
    Portuguese,
    Spanish,
}

impl Language {
    pub fn from_u8(language_code: u8) -> Result<Self, crate::Error> {
        Ok(match language_code {
            1 => Language::English,
            2 => Language::ChineseSimplified,
            3 => Language::ChineseTraditional,
            4 => Language::Czech,
            5 => Language::French,
            6 => Language::Italian,
            7 => Language::Japanese,
            8 => Language::Korean,
            9 => Language::Portuguese,
            10 => Language::Spanish,
            _ => return Err(crate::Error::UnknownLanguage),
        })
    }
}

#[derive(Debug, Clone)]
pub enum WordlistWrapper {
    English(coins_bip39::English),
    ChineseSimplified(coins_bip39::ChineseSimplified),
    ChineseTraditional(coins_bip39::ChineseTraditional),
    Czech(coins_bip39::Czech),
    French(coins_bip39::French),
    Italian(coins_bip39::Italian),
    Japanese(coins_bip39::Japanese),
    Korean(coins_bip39::Korean),
    Portuguese(coins_bip39::Portuguese),
    Spanish(coins_bip39::Spanish),
}

pub enum QueryMode {
    StartsWith,
    Contains,
}

impl QueryMode {
    pub fn from_u8(mode: u8) -> Result<Self, crate::Error> {
        Ok(match mode {
            1 => QueryMode::StartsWith,
            2 => QueryMode::Contains,
            _ => return Err(crate::Error::UnknownQueryMode),
        })
    }
}

impl WordlistWrapper {
    pub fn new(lang: u8) -> Result<WordlistWrapper, crate::Error> {
        let language = Language::from_u8(lang)?;
        Ok(language.gen_wordlist_wrapper())
    }

    pub fn get_all(&self) -> &'static [&'static str] {
        use coins_bip39::Wordlist as _;
        match self {
            WordlistWrapper::English(_) => coins_bip39::English::get_all(),
            WordlistWrapper::ChineseSimplified(_) => coins_bip39::ChineseSimplified::get_all(),
            WordlistWrapper::ChineseTraditional(_) => coins_bip39::ChineseTraditional::get_all(),
            WordlistWrapper::Czech(_) => coins_bip39::Czech::get_all(),
            WordlistWrapper::French(_) => coins_bip39::French::get_all(),
            WordlistWrapper::Italian(_) => coins_bip39::Italian::get_all(),
            WordlistWrapper::Japanese(_) => coins_bip39::Japanese::get_all(),
            WordlistWrapper::Korean(_) => coins_bip39::Korean::get_all(),
            WordlistWrapper::Portuguese(_) => coins_bip39::Portuguese::get_all(),
            WordlistWrapper::Spanish(_) => coins_bip39::Spanish::get_all(),
        }
    }

    pub fn query_phrase(self, keyword: &str, mode: QueryMode) -> Vec<String> {
        let all_words = self.get_all();
        let keyword = keyword.to_lowercase();
        all_words
            .iter()
            .filter(|word| match mode {
                QueryMode::StartsWith => word.to_lowercase().starts_with(&keyword),
                QueryMode::Contains => word.to_lowercase().contains(&keyword),
            })
            .map(|word| word.to_string())
            .collect()
    }

    pub fn exact_query_phrase(&self, phrase: &str) -> Option<String> {
        let all_words = self.get_all();
        let keyword = phrase.to_lowercase();
        all_words
            .iter()
            .find(|word| word.to_lowercase() == keyword)
            .map(|word| word.to_string())
    }
}

impl Language {
    pub fn gen_wordlist_wrapper(self) -> WordlistWrapper {
        match self {
            Language::English => WordlistWrapper::English(coins_bip39::English),
            Language::ChineseSimplified => {
                WordlistWrapper::ChineseSimplified(coins_bip39::ChineseSimplified)
            }
            Language::ChineseTraditional => {
                WordlistWrapper::ChineseTraditional(coins_bip39::ChineseTraditional)
            }
            Language::Czech => WordlistWrapper::Czech(coins_bip39::Czech),
            Language::French => WordlistWrapper::French(coins_bip39::French),
            Language::Italian => WordlistWrapper::Italian(coins_bip39::Italian),
            Language::Japanese => WordlistWrapper::Japanese(coins_bip39::Japanese),
            Language::Korean => WordlistWrapper::Korean(coins_bip39::Korean),
            Language::Portuguese => WordlistWrapper::Portuguese(coins_bip39::Portuguese),
            Language::Spanish => WordlistWrapper::Spanish(coins_bip39::Spanish),
        }
    }

    pub fn gen_phrase(self, count: usize) -> Result<Vec<String>, crate::Error> {
        let mut rng = rand::thread_rng();
        let phrase =
            match self {
                Language::English => {
                    coins_bip39::Mnemonic::<coins_bip39::English>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::ChineseSimplified => coins_bip39::Mnemonic::<
                    coins_bip39::ChineseSimplified,
                >::new_with_count(&mut rng, count)?
                .to_phrase(),
                Language::ChineseTraditional => coins_bip39::Mnemonic::<
                    coins_bip39::ChineseTraditional,
                >::new_with_count(&mut rng, count)?
                .to_phrase(),
                Language::Czech => {
                    coins_bip39::Mnemonic::<coins_bip39::Czech>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::French => {
                    coins_bip39::Mnemonic::<coins_bip39::French>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::Italian => {
                    coins_bip39::Mnemonic::<coins_bip39::Italian>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::Japanese => {
                    coins_bip39::Mnemonic::<coins_bip39::Japanese>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::Korean => {
                    coins_bip39::Mnemonic::<coins_bip39::Korean>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
                Language::Portuguese => {
                    coins_bip39::Mnemonic::<coins_bip39::Portuguese>::new_with_count(
                        &mut rng, count,
                    )?
                    .to_phrase()
                }
                Language::Spanish => {
                    coins_bip39::Mnemonic::<coins_bip39::Spanish>::new_with_count(&mut rng, count)?
                        .to_phrase()
                }
            };
        let words = phrase.split(' ').map(|word| word.to_string()).collect();
        Ok(words)
    }
}
