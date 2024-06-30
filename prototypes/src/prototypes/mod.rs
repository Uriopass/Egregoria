// Find template in _template.rs
crate::gen_prototypes!(
    mod item:          ItemID              = ItemPrototype,
    mod building:      BuildingPrototypeID = BuildingPrototype,
    mod goods_company: GoodsCompanyID      = GoodsCompanyPrototype => BuildingPrototypeID,
    mod leisure:       LeisurePrototypeID  = LeisurePrototype => BuildingPrototypeID,
    mod solar:         SolarPanelID        = SolarPanelPrototype => GoodsCompanyID,

    mod vehicle:       VehiclePrototypeID = VehiclePrototype,
    mod road_vehicle:  RoadVehicleID      = RoadVehiclePrototype => VehiclePrototypeID,
    mod rolling_stock: RollingStockID     = RollingStockPrototype => VehiclePrototypeID,

    mod colors:         ColorsPrototypeID   = ColorsPrototype,
    mod freightstation: FreightStationPrototypeID = FreightStationPrototype,
);

mod base;
pub use base::*;
