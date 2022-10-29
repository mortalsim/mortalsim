use crate::blood::{BloodVessel, ClosedCirculatorySystem, ClosedCirculationManager};

pub type HumanCirculatorySystem = ClosedCirculatorySystem<HumanBloodVessel>;
pub type HumanBloodManager = ClosedCirculationManager<HumanBloodVessel>;

pub const HUMAN_CIRCULATION_FILEPATH: &str = "config/circulation/human_circulation.json";

impl HumanCirculatorySystem {
    pub fn new() -> HumanCirculatorySystem {
        match HumanCirculatorySystem::from_json_file(HUMAN_CIRCULATION_FILEPATH) {
            Err(err) => panic!("Error loading Human Circulatory System from '{}': {}", HUMAN_CIRCULATION_FILEPATH, err),
            Ok(circ) => circ
        }
    }
}

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum HumanBloodVessel {
    Aorta,
    RightBraciocephalicArtery,
    RightSubclavianArtery,
    RightAxillaryArtery,
    RightBrachialArtery,
    RightUlnarArtery,
    RightRadialArtery,
    RightCommonCarotidArtery,
    RightCarotidSinusArtery,
    RightInternalCarotidArtery,
    RightExternalCarotidArtery,
    LeftSubclavianArtery,
    LeftAxillaryArtery,
    LeftBrachialArtery,
    LeftUlnarArtery,
    LeftRadialArtery,
    LeftCommonCarotidArtery,
    LeftCarotidSinusArtery,
    LeftInternalCarotidArtery,
    LeftExternalCarotidArtery,
    ThoracicAorta,
    AbdominalAorta,
    CeliacArtery,
    CommonHepaticArtery,
    RightGastricArtery,
    SplenicArtery,
    LeftGastricArtery,
    SuperiorMesentericArtery,
    InferiorMesentericArtery,
    RightRenalArtery,
    LeftRenalArtery,
    RightCommonIliacArtery,
    RightInternalIliacArtery,
    RightExternalIliacArtery,
    RightCommonFemoralArtery,
    RightDeepFemoralArtery,
    RightSuperficialFemoralArtery,
    RightPoplitealArtery,
    RightAnteriorTibialArtery,
    RightPosteriorTibialArtery,
    RightFibularArtery,
    LeftCommonIliacArtery,
    LeftInternalIliacArtery,
    LeftExternalIliacArtery,
    LeftCommonFemoralArtery,
    LeftDeepFemoralArtery,
    LeftSuperficialFemoralArtery,
    LeftPoplitealArtery,
    LeftAnteriorTibialArtery,
    LeftPosteriorTibialArtery,
    LeftFibularArtery,
    VenaCava,
    SuperiorVenaCava,
    RightBrachiocephalicVein,
    RightSubclavianVein,
    RightAxillaryVein,
    RightBasilicVein,
    RightCephalicVein,
    RightInternalJugularVein,
    LeftBrachiocephalicVein,
    LeftSubclavianVein,
    LeftAxillaryVein,
    LeftBasilicVein,
    LeftCephalicVein,
    LeftInternalJugularVein,
    InferiorVenaCava,
    HepaticVein,
    SplenicVein,
    SuperiorMesentericVein,
    InferiorMesentericVein,
    LeftGastricVein,
    RightGastricVein,
    LeftRenalVein,
    RightRenalVein,
    RightCommonIliacVein,
    RightInternalIliacVein,
    RightExternalIliacVein,
    RightDeepFemoralVein,
    RightGreatSaphenousVein,
    RightFemoralVein,
    RightPoplitealVein,
    RightSmallSaphenousVein,
    RightAnteriorTibialVein,
    RightPosteriorTibialVein,
    LeftCommonIliacVein,
    LeftInternalIliacVein,
    LeftExternalIliacVein,
    LeftDeepFemoralVein,
    LeftGreatSaphenousVein,
    LeftFemoralVein,
    LeftPoplitealVein,
    LeftSmallSaphenousVein,
    LeftAnteriorTibialVein,
    LeftPosteriorTibialVein,
}

impl BloodVessel for HumanBloodVessel {
    fn source() -> HumanBloodVessel {
        Self::Aorta
    }
    fn sink() -> HumanBloodVessel {
        Self::VenaCava
    }
}
