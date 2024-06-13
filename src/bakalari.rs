use rezvrh_scraper::Bakalari;

pub struct BakaWrapper {
    bakalari: Bakalari,
}

impl BakaWrapper {
    pub const fn new(bakalari: Bakalari) -> Self {
        Self { bakalari }
    }

    pub const fn get_light(&self) -> u8 {
        todo!()
    }
}
