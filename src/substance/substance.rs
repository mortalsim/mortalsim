
/*
 * THIS FILE IS AUTOMATICALLY GENERATED.
 * SOURCE: scripts/substance_writer.js
 */

use std::fmt;
use crate::units::chemical::MolarMass;
use crate::units::mechanical::Density;

/// Enumeration of chemical substances.
/// These are typically named as their most abundant form in biological contexts.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Substance {
    /// Adenosine Diphosphate (ADP)
    ADP,
    /// Adenosine Monophosphate (AMP)
    AMP,
    /// Adenosine Triphosphate (ATP)
    ATP,
    /// Calcium (Ca2+)
    Ca,
    /// Carbon Dioxide (CO2)
    CO2,
    /// Chloride (Cl-)
    Cl,
    /// Alpha D Glucose (GLC)
    GLC,
    /// L-Glucose (GLCL)
    GLCL,
    /// Fructose (FRC)
    FRC,
    /// Hydrogen (H+)
    H,
    /// Water (H2O)
    H2O,
    /// Potassium (K+)
    K,
    /// Lactate (LAC)
    LAC,
    /// Lactate Dehydrogenase (LDH)
    LDH,
    /// Dinitrogen (N2)
    N2,
    /// Nicotinamide Adenine Dinucleotide (NAD+)
    NAD,
    /// Reduced NAD (NADH)
    NADH,
    /// Sodium (Na+)
    Na,
    /// Salt (NaCl)
    NaCl,
    /// Dioxygen (O2)
    O2,
    /// Phosphofructokinase (PFK)
    PFK,
    /// Phosphoglycerate Kinase (PGK)
    PGK,
    /// Pyruvate (PYR)
    PYR,
    /// Monosodium Glutamate (MSG)
    MSG,
    /// Ammonia (NH3)
    NH3,
    /// Bleach (NaClO)
    NaClO,
    /// Alanine (ALA)
    ALA,
    /// Arginine (ARG+)
    ARG,
    /// Asparagine (ASN)
    ASN,
    /// Aspartic Acid (ASP-)
    ASP,
    /// Cysteine (CYS)
    CYS,
    /// Glutamine (GLN)
    GLN,
    /// Glutamic Acid (GLU-)
    GLU,
    /// Glycine (GLY)
    GLY,
    /// Histidine (HIS+)
    HIS,
    /// Isoleucine (ILE)
    ILE,
    /// Leucine (LEU)
    LEU,
    /// Lysine (LYS+)
    LYS,
    /// Methionine (MET)
    MET,
    /// Phenylalanine (PHE)
    PHE,
    /// Proline (PRO)
    PRO,
    /// Serine (SER)
    SER,
    /// Threonine (THR)
    THR,
    /// Tryptophan (TRP)
    TRP,
    /// Tyrosine (TYR)
    TYR,
    /// Valine (VAL)
    VAL,
    /// Vitamin A (Retinol)
    Retinol,
    /// Vitamin A Aldehyde (Retinal)
    Retinal,
    /// Vitamin A Acid (RetinoicAcid)
    RetinoicAcid,
    /// Vitamin B1 (Thiamine+)
    Thiamine,
    /// Vitamin B2 (Riboflavin)
    Riboflavin,
    /// Vitamin B3 (Niacin)
    Niacin,
    /// Vitamin B5 (PantothenicAcid)
    PantothenicAcid,
    /// Vitamin B6 (Pyridoxine)
    Pyridoxine,
    /// Vitamin B7 (Biotin)
    Biotin,
    /// Vitamin B9 (Folate)
    Folate,
    /// Vitamin B12a (HdxCbl)
    HdxCbl,
    /// Coenzyme B12 (AdoCbl)
    AdoCbl,
    /// Methylcobalamin B12 (MeCbl)
    MeCbl,
    /// Cyanocobalamin B12 (CynCbl)
    CynCbl,
    /// Vitamin C (AscorbicAcid)
    AscorbicAcid,
    /// Vitamin D2 (Ergocalciferol)
    Ergocalciferol,
    /// Vitamin D3 (Cholecalciferol)
    Cholecalciferol,
    /// Vitamin E (AlphaTocopherol)
    AlphaTocopherol,
    /// Vitamin E (BetaTocopherol)
    BetaTocopherol,
    /// Vitamin E (DeltaTocopherol)
    DeltaTocopherol,
    /// Vitamin E (GammaTocopherol)
    GammaTocopherol,
    /// Vitamin E (AlphaTocotrienol)
    AlphaTocotrienol,
    /// Vitamin E (BetaTocotrienol)
    BetaTocotrienol,
    /// Vitamin E (DeltaTocotrienol)
    DeltaTocotrienol,
    /// Vitamin E (GammaTocotrienol)
    GammaTocotrienol,
    /// Vitamin K1 (Phytomenadione)
    Phytomenadione,
    /// Vitamin K2-4 (MK4)
    MK4,
    /// Vitamin K3 (Menadione)
    Menadione,
    /// Starch (Amylose)
    Amylose,
    /// Starch (Amylopectin)
    Amylopectin,
    /// Cellulose (Cellulose)
    Cellulose,
    /// Glycogen (Glycogen)
    Glycogen,

}

impl fmt::Display for Substance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let charge = self.charge();
        if charge == 1 {
            write!(f, "{} ({:?}+)", self.name(), self)
        } else if charge > 1 {
            write!(f, "{} ({:?} {}+)", self.name(), self, charge)
        } else if charge == -1 {
            write!(f, "{} ({:?}-)", self.name(), self)
        } else if charge < -1 {
            write!(f, "{} ({:?} {}-)", self.name(), self, charge)
        } else {
            write!(f, "{} ({:?})", self.name(), self)
        }
    }
}

impl Substance {
    /// Full substance name
    pub fn name(&self) -> &'static str {
        match self {
            Self::ADP => "Adenosine Diphosphate",
            Self::AMP => "Adenosine Monophosphate",
            Self::ATP => "Adenosine Triphosphate",
            Self::Ca => "Calcium",
            Self::CO2 => "Carbon Dioxide",
            Self::Cl => "Chloride",
            Self::GLC => "Alpha D Glucose",
            Self::GLCL => "L-Glucose",
            Self::FRC => "Fructose",
            Self::H => "Hydrogen",
            Self::H2O => "Water",
            Self::K => "Potassium",
            Self::LAC => "Lactate",
            Self::LDH => "Lactate Dehydrogenase",
            Self::N2 => "Dinitrogen",
            Self::NAD => "Nicotinamide Adenine Dinucleotide",
            Self::NADH => "Reduced NAD",
            Self::Na => "Sodium",
            Self::NaCl => "Salt",
            Self::O2 => "Dioxygen",
            Self::PFK => "Phosphofructokinase",
            Self::PGK => "Phosphoglycerate Kinase",
            Self::PYR => "Pyruvate",
            Self::MSG => "Monosodium Glutamate",
            Self::NH3 => "Ammonia",
            Self::NaClO => "Bleach",
            Self::ALA => "Alanine",
            Self::ARG => "Arginine",
            Self::ASN => "Asparagine",
            Self::ASP => "Aspartic Acid",
            Self::CYS => "Cysteine",
            Self::GLN => "Glutamine",
            Self::GLU => "Glutamic Acid",
            Self::GLY => "Glycine",
            Self::HIS => "Histidine",
            Self::ILE => "Isoleucine",
            Self::LEU => "Leucine",
            Self::LYS => "Lysine",
            Self::MET => "Methionine",
            Self::PHE => "Phenylalanine",
            Self::PRO => "Proline",
            Self::SER => "Serine",
            Self::THR => "Threonine",
            Self::TRP => "Tryptophan",
            Self::TYR => "Tyrosine",
            Self::VAL => "Valine",
            Self::Retinol => "Vitamin A",
            Self::Retinal => "Vitamin A Aldehyde",
            Self::RetinoicAcid => "Vitamin A Acid",
            Self::Thiamine => "Vitamin B1",
            Self::Riboflavin => "Vitamin B2",
            Self::Niacin => "Vitamin B3",
            Self::PantothenicAcid => "Vitamin B5",
            Self::Pyridoxine => "Vitamin B6",
            Self::Biotin => "Vitamin B7",
            Self::Folate => "Vitamin B9",
            Self::HdxCbl => "Vitamin B12a",
            Self::AdoCbl => "Coenzyme B12",
            Self::MeCbl => "Methylcobalamin B12",
            Self::CynCbl => "Cyanocobalamin B12",
            Self::AscorbicAcid => "Vitamin C",
            Self::Ergocalciferol => "Vitamin D2",
            Self::Cholecalciferol => "Vitamin D3",
            Self::AlphaTocopherol => "Vitamin E",
            Self::BetaTocopherol => "Vitamin E",
            Self::DeltaTocopherol => "Vitamin E",
            Self::GammaTocopherol => "Vitamin E",
            Self::AlphaTocotrienol => "Vitamin E",
            Self::BetaTocotrienol => "Vitamin E",
            Self::DeltaTocotrienol => "Vitamin E",
            Self::GammaTocotrienol => "Vitamin E",
            Self::Phytomenadione => "Vitamin K1",
            Self::MK4 => "Vitamin K2-4",
            Self::Menadione => "Vitamin K3",
            Self::Amylose => "Starch",
            Self::Amylopectin => "Starch",
            Self::Cellulose => "Cellulose",
            Self::Glycogen => "Glycogen",

        }
    }

    /// Overall substance charge
    pub fn charge(&self) -> i8 {
        match self {
            Self::ADP => 0,
            Self::AMP => 0,
            Self::ATP => 0,
            Self::Ca => 2,
            Self::CO2 => 0,
            Self::Cl => -1,
            Self::GLC => 0,
            Self::GLCL => 0,
            Self::FRC => 0,
            Self::H => 1,
            Self::H2O => 0,
            Self::K => 1,
            Self::LAC => 0,
            Self::LDH => 0,
            Self::N2 => 0,
            Self::NAD => 1,
            Self::NADH => 0,
            Self::Na => 1,
            Self::NaCl => 0,
            Self::O2 => 0,
            Self::PFK => 0,
            Self::PGK => 0,
            Self::PYR => 0,
            Self::MSG => 0,
            Self::NH3 => 0,
            Self::NaClO => 0,
            Self::ALA => 0,
            Self::ARG => 1,
            Self::ASN => 0,
            Self::ASP => -1,
            Self::CYS => 0,
            Self::GLN => 0,
            Self::GLU => -1,
            Self::GLY => 0,
            Self::HIS => 1,
            Self::ILE => 0,
            Self::LEU => 0,
            Self::LYS => 1,
            Self::MET => 0,
            Self::PHE => 0,
            Self::PRO => 0,
            Self::SER => 0,
            Self::THR => 0,
            Self::TRP => 0,
            Self::TYR => 0,
            Self::VAL => 0,
            Self::Retinol => 0,
            Self::Retinal => 0,
            Self::RetinoicAcid => 0,
            Self::Thiamine => 1,
            Self::Riboflavin => 0,
            Self::Niacin => 0,
            Self::PantothenicAcid => 0,
            Self::Pyridoxine => 0,
            Self::Biotin => 0,
            Self::Folate => 0,
            Self::HdxCbl => 0,
            Self::AdoCbl => 0,
            Self::MeCbl => 0,
            Self::CynCbl => 0,
            Self::AscorbicAcid => 0,
            Self::Ergocalciferol => 0,
            Self::Cholecalciferol => 0,
            Self::AlphaTocopherol => 0,
            Self::BetaTocopherol => 0,
            Self::DeltaTocopherol => 0,
            Self::GammaTocopherol => 0,
            Self::AlphaTocotrienol => 0,
            Self::BetaTocotrienol => 0,
            Self::DeltaTocotrienol => 0,
            Self::GammaTocotrienol => 0,
            Self::Phytomenadione => 0,
            Self::MK4 => 0,
            Self::Menadione => 0,
            Self::Amylose => 0,
            Self::Amylopectin => 0,
            Self::Cellulose => 0,
            Self::Glycogen => 0,

        }
    }

    /// Typical molar mass of the substance
    pub fn molar_mass(&self) -> MolarMass<f64> {
        match self {
            Self::ADP => MolarMass::from_gpmol(427.201),
            Self::AMP => MolarMass::from_gpmol(347.2212),
            Self::ATP => MolarMass::from_gpmol(507.18),
            Self::Ca => MolarMass::from_gpmol(40.078),
            Self::CO2 => MolarMass::from_gpmol(44.01),
            Self::Cl => MolarMass::from_gpmol(35.453),
            Self::GLC => MolarMass::from_gpmol(180.156),
            Self::GLCL => MolarMass::from_gpmol(180.156),
            Self::FRC => MolarMass::from_gpmol(180.156),
            Self::H => MolarMass::from_gpmol(1.00794),
            Self::H2O => MolarMass::from_gpmol(18.0153),
            Self::K => MolarMass::from_gpmol(39.0983),
            Self::LAC => MolarMass::from_gpmol(89.07),
            Self::LDH => MolarMass::from_gpmol(144000.0),
            Self::N2 => MolarMass::from_gpmol(28.0134),
            Self::NAD => MolarMass::from_gpmol(663.43),
            Self::NADH => MolarMass::from_gpmol(665.125),
            Self::Na => MolarMass::from_gpmol(22.989769),
            Self::NaCl => MolarMass::from_gpmol(58.44),
            Self::O2 => MolarMass::from_gpmol(31.9988),
            Self::PFK => MolarMass::from_gpmol(85000.0),
            Self::PGK => MolarMass::from_gpmol(45000.0),
            Self::PYR => MolarMass::from_gpmol(88.06),
            Self::MSG => MolarMass::from_gpmol(88.06),
            Self::NH3 => MolarMass::from_gpmol(17.031),
            Self::NaClO => MolarMass::from_gpmol(74.44),
            Self::ALA => MolarMass::from_gpmol(89.1),
            Self::ARG => MolarMass::from_gpmol(174.2),
            Self::ASN => MolarMass::from_gpmol(132.1),
            Self::ASP => MolarMass::from_gpmol(133.1),
            Self::CYS => MolarMass::from_gpmol(121.6),
            Self::GLN => MolarMass::from_gpmol(146.1),
            Self::GLU => MolarMass::from_gpmol(147.1),
            Self::GLY => MolarMass::from_gpmol(75.1),
            Self::HIS => MolarMass::from_gpmol(155.2),
            Self::ILE => MolarMass::from_gpmol(131.2),
            Self::LEU => MolarMass::from_gpmol(131.2),
            Self::LYS => MolarMass::from_gpmol(146.2),
            Self::MET => MolarMass::from_gpmol(149.2),
            Self::PHE => MolarMass::from_gpmol(165.2),
            Self::PRO => MolarMass::from_gpmol(115.1),
            Self::SER => MolarMass::from_gpmol(105.1),
            Self::THR => MolarMass::from_gpmol(119.2),
            Self::TRP => MolarMass::from_gpmol(204.2),
            Self::TYR => MolarMass::from_gpmol(181.2),
            Self::VAL => MolarMass::from_gpmol(117.5),
            Self::Retinol => MolarMass::from_gpmol(286.459),
            Self::Retinal => MolarMass::from_gpmol(284.443),
            Self::RetinoicAcid => MolarMass::from_gpmol(300.43512),
            Self::Thiamine => MolarMass::from_gpmol(265.36),
            Self::Riboflavin => MolarMass::from_gpmol(376.369),
            Self::Niacin => MolarMass::from_gpmol(123.111),
            Self::PantothenicAcid => MolarMass::from_gpmol(219.237),
            Self::Pyridoxine => MolarMass::from_gpmol(169.18),
            Self::Biotin => MolarMass::from_gpmol(244.31),
            Self::Folate => MolarMass::from_gpmol(441.404),
            Self::HdxCbl => MolarMass::from_gpmol(1346.377),
            Self::AdoCbl => MolarMass::from_gpmol(1579.608),
            Self::MeCbl => MolarMass::from_gpmol(1344.405),
            Self::CynCbl => MolarMass::from_gpmol(1355.388),
            Self::AscorbicAcid => MolarMass::from_gpmol(176.124),
            Self::Ergocalciferol => MolarMass::from_gpmol(396.659),
            Self::Cholecalciferol => MolarMass::from_gpmol(384.648),
            Self::AlphaTocopherol => MolarMass::from_gpmol(430.71),
            Self::BetaTocopherol => MolarMass::from_gpmol(416.68),
            Self::DeltaTocopherol => MolarMass::from_gpmol(402.65),
            Self::GammaTocopherol => MolarMass::from_gpmol(416.68),
            Self::AlphaTocotrienol => MolarMass::from_gpmol(424.7),
            Self::BetaTocotrienol => MolarMass::from_gpmol(410.6),
            Self::DeltaTocotrienol => MolarMass::from_gpmol(396.6),
            Self::GammaTocotrienol => MolarMass::from_gpmol(410.6),
            Self::Phytomenadione => MolarMass::from_gpmol(450.707),
            Self::MK4 => MolarMass::from_gpmol(444.659),
            Self::Menadione => MolarMass::from_gpmol(172.183),
            Self::Amylose => MolarMass::from_gpmol(100000.0),
            Self::Amylopectin => MolarMass::from_gpmol(1500000.0),
            Self::Cellulose => MolarMass::from_gpmol(162000.0),
            Self::Glycogen => MolarMass::from_gpmol(5404680.0),

        }
    }

    /// Typical density of the substance
    pub fn density(&self) -> Density<f64> {
        match self {
            Self::ADP => Density::from_gpcc(2.49),
            Self::AMP => Density::from_gpcc(1.04),
            Self::ATP => Density::from_gpcc(1.04),
            Self::Ca => Density::from_gpcc(1.55),
            Self::CO2 => Density::from_gpcc(0.001977),
            Self::Cl => Density::from_gpcc(0.0032),
            Self::GLC => Density::from_gpcc(1.54),
            Self::GLCL => Density::from_gpcc(1.54),
            Self::FRC => Density::from_gpcc(1.694),
            Self::H => Density::from_gpcc(0.00008988),
            Self::H2O => Density::from_gpcc(1.0),
            Self::K => Density::from_gpcc(0.862),
            Self::LAC => Density::from_gpcc(1.21),
            Self::LDH => Density::from_gpcc(1.43),
            Self::N2 => Density::from_gpcc(0.001251),
            Self::NAD => Density::from_gpcc(1.578),
            Self::NADH => Density::from_gpcc(1.578),
            Self::Na => Density::from_gpcc(0.97),
            Self::NaCl => Density::from_gpcc(2.16),
            Self::O2 => Density::from_gpcc(0.001429),
            Self::PFK => Density::from_gpcc(1.5625),
            Self::PGK => Density::from_gpcc(1.642),
            Self::PYR => Density::from_gpcc(1.25),
            Self::MSG => Density::from_gpcc(1.52),
            Self::NH3 => Density::from_gpcc(0.73),
            Self::NaClO => Density::from_gpcc(1.11),
            Self::ALA => Density::from_gpcc(1.42),
            Self::ARG => Density::from_gpcc(1.48),
            Self::ASN => Density::from_gpcc(1.53),
            Self::ASP => Density::from_gpcc(1.66),
            Self::CYS => Density::from_gpcc(1.92),
            Self::GLN => Density::from_gpcc(1.36),
            Self::GLU => Density::from_gpcc(1.54),
            Self::GLY => Density::from_gpcc(1.6),
            Self::HIS => Density::from_gpcc(1.49),
            Self::ILE => Density::from_gpcc(1.34),
            Self::LEU => Density::from_gpcc(1.34),
            Self::LYS => Density::from_gpcc(1.36),
            Self::MET => Density::from_gpcc(1.34),
            Self::PHE => Density::from_gpcc(1.08),
            Self::PRO => Density::from_gpcc(1.36),
            Self::SER => Density::from_gpcc(1.48),
            Self::THR => Density::from_gpcc(1.31),
            Self::TRP => Density::from_gpcc(1.2),
            Self::TYR => Density::from_gpcc(1.18),
            Self::VAL => Density::from_gpcc(1.23),
            Self::Retinol => Density::from_gpcc(0.944),
            Self::Retinal => Density::from_gpcc(0.97),
            Self::RetinoicAcid => Density::from_gpcc(1.06),
            Self::Thiamine => Density::from_gpcc(1.24),
            Self::Riboflavin => Density::from_gpcc(1.454),
            Self::Niacin => Density::from_gpcc(1.473),
            Self::PantothenicAcid => Density::from_gpcc(1.213),
            Self::Pyridoxine => Density::from_gpcc(1.626),
            Self::Biotin => Density::from_gpcc(1.213),
            Self::Folate => Density::from_gpcc(1.77),
            Self::HdxCbl => Density::from_gpcc(1.9),
            Self::AdoCbl => Density::from_gpcc(1.12),
            Self::MeCbl => Density::from_gpcc(1.37),
            Self::CynCbl => Density::from_gpcc(1.53),
            Self::AscorbicAcid => Density::from_gpcc(1.65),
            Self::Ergocalciferol => Density::from_gpcc(0.94),
            Self::Cholecalciferol => Density::from_gpcc(0.96),
            Self::AlphaTocopherol => Density::from_gpcc(0.95),
            Self::BetaTocopherol => Density::from_gpcc(0.95),
            Self::DeltaTocopherol => Density::from_gpcc(0.96),
            Self::GammaTocopherol => Density::from_gpcc(0.95),
            Self::AlphaTocotrienol => Density::from_gpcc(0.88),
            Self::BetaTocotrienol => Density::from_gpcc(0.89),
            Self::DeltaTocotrienol => Density::from_gpcc(0.91),
            Self::GammaTocotrienol => Density::from_gpcc(0.89),
            Self::Phytomenadione => Density::from_gpcc(0.989),
            Self::MK4 => Density::from_gpcc(1.1),
            Self::Menadione => Density::from_gpcc(1.28),
            Self::Amylose => Density::from_gpcc(1.5),
            Self::Amylopectin => Density::from_gpcc(1.6),
            Self::Cellulose => Density::from_gpcc(1.5),
            Self::Glycogen => Density::from_gpcc(1.6),

        }
    }

    /// Typical molar volume of the substance
    pub fn molar_volume(&self) -> crate::units::chemical::MolarVolume<f64> {
        self.molar_mass() / self.density()
    }
}
