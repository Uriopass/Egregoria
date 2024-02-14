use crate::uiworld::UiWorld;

pub fn train_properties(_uiw: &UiWorld) {}

/*
if ui.button(freightstation).clicked() {
   *uiworld.write::<Tool>() = Tool::SpecialBuilding;

   uiworld.write::<SpecialBuildingResource>().opt = Some(SpecialBuildKind {
       make: Box::new(move |args| {
           let obb = args.obb;
           let c = obb.center().z(args.mpos.z + 0.3);

           let [offx, offy] = obb.axis().map(|x| x.normalize().z(0.0));

           let pat =
               LanePatternBuilder::new().rail(true).one_way(true).build();

           let mut commands = Vec::with_capacity(5);

           commands.push(WorldCommand::MapMakeConnection {
               from: MapProject::ground(c - offx * 45.0 - offy * 100.0),
               to: MapProject::ground(c - offx * 45.0 + offy * 100.0),
               inter: None,
               pat,
           });

           commands.push(WorldCommand::MapBuildSpecialBuilding {
               pos: args.obb,
               kind: BuildingKind::RailFreightStation(proto.id),
               gen: BuildingGen::NoWalkway {
                   door_pos: Vec2::ZERO,
               },
               zone: None,
               connected_road: args.connected_road,
           });
           commands
       }),
       size: proto.size,
       asset: proto.asset.clone(),
       road_snap: false,
   });
}
*/
