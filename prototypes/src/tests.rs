#![cfg(test)]

use crate::load::load_prototypes;
use crate::{try_prototype, GoodsCompanyID, ItemID, SolarPanelID};

#[test]
fn test_base() {
    unsafe {
        match load_prototypes("../") {
            Ok(_) => {}
            Err(e) => {
                println!("failed to load prototypes: {}", e);
                assert!(false);
            }
        }

        println!(
            "{:?}",
            try_prototype(ItemID::new("job-opening"))
                .unwrap()
                .optout_exttrade
        );
        println!("{:?}", try_prototype(ItemID::new("cereal")));
        println!("{:#?}", try_prototype(GoodsCompanyID::new("bakery")));
        println!("{:?}", ItemID::new("unknown"));
        println!("{:?}", try_prototype(GoodsCompanyID::new("solar-panel")));
        println!("{:?}", try_prototype(SolarPanelID::new("solar-panel")));
    }
}
