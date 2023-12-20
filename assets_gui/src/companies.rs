use common::descriptions::GoodsCompanyDescriptionJSON;
use common::saveload::Encoder;
use std::io;

pub struct Companies {
    pub companies: Vec<GoodsCompanyDescriptionJSON>,
    pub changed: bool,
}

impl Companies {
    pub fn new() -> io::Result<Self> {
        let cjson = common::saveload::load_raw("assets/companies.json")?;
        let companies = common::saveload::JSONPretty::decode(&cjson)?;
        Ok(Self {
            companies,
            changed: false,
        })
    }

    #[allow(dead_code)]
    pub fn save(&self) {
        common::saveload::JSONPretty::save(&self.companies, "companies");
    }
}
