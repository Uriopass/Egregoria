#![cfg(test)]

use crate::{load_prototypes, try_prototype, GoodsCompanyID, ItemID};

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
    }
}
