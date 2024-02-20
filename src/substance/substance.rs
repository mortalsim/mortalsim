use crate::units::chemical::MolarMass;
use std::fmt;

/// Enumeration of chemical substances.
/// These are typically named as their most abundant form in nature.
/// Variations are suffixed with appropriate identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Substance {
    ADP,  // Adenosine Diphosphate (ADP)
    AMP,  // Adenosine Monophosphate (AMP)
    ATP,  // Adenosine Triphosphate (ATP)
    Ca,   // Calcium (Ca 2+)
    CO2,  // Carbon Dioxide (CO2)
    Cl,   // Chloride (Cl-)
    GLC,  // Alpha D Glucose (GLC)
    GLCL, // L-Glucose (GLCL)
    H,    // Hydrogen (H+)
    H2O,  // Water (H2O)
    K,    // Potassium (K+)
    LAC,  // Lactate (LAC)
    LDH,  // Lactate Dehydrogenase (LDH)
    MSG,  // Monosodium Glutamate (MSG)
    N2,   // Dinitrogen (N2)
    NAD,  // Nicotinamide Adenine Dinucleotide (NAD+)
    NADH, // Reduced NAD (NADH)
    Na,   // Sodium (Na+)
    NaCl, // Salt (NaCl)
    O2,   // Dioxygen (O2)
    PFK,  // Phosphofructokinase (PFK)
    PGK,  // Phosphoglycerate Kinase (PGK)
    PYR,  // Pyruvate (PYR)

    // Amino acids
    ALA, // Alanine 
    ARG, // Arginine 
    ASN, // Asparagine 
    ASP, // Aspartic Acid 
    CYS, // Cysteine 
    GLN, // Glutamine 
    GLU, // Glutamic Acid
    GLY, // Glycine
    HIS, // Histidine 
    ILE, // Isoleucine 
    LEU, // Leucine 
    LYS, // Lysine 
    MET, // Methionine 
    PHE, // Phenylalanine 
    PRO, // Proline 
    SER, // Serine 
    THR, // Threonine 
    TRP, // Tryptophan 
    TYR, // Tyrosine 
    VAL, // Valine 

    // Starch
    AML(u16), // Amylose with avg chain length
    AMN(u16), // Amylopectin with avg total chain length

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
            Self::H => "Hydrogen",
            Self::H2O => "Water",
            Self::K => "Potassium",
            Self::LAC => "Lactate",
            Self::LDH => "Lactate Dehydrogenase",
            Self::MSG => "Monosodium Glutamate",
            Self::N2 => "Dinitrogen",
            Self::NAD => "Nicotinamide Adenine Dinucleotide",
            Self::NADH => "Reduced Nicotinamide Adenine Dinucleotide",
            Self::Na => "Sodium",
            Self::NaCl => "Salt",
            Self::O2 => "Dioxygen",
            Self::PFK => "Phosphofructokinase",
            Self::PGK => "Phosphoglycerate Kinase",
            Self::PYR => "Pyruvate",

            // Amino Acids
            Self::ALA => "Alanine",
            Self::ARG => "Arginine",
            Self::ASN => "Asparagine",
            Self::ASP => "Aspartic Acid",
            Self::CYS => "Cysteine",
            Self::GLN => "Glutamine",
            Self::GLU => "Glutamic Aci",
            Self::GLY => "Glycin",
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

            // Starch
            Self::AML(_) => "Amylose",
            Self::AMN(_) => "Amylopectin",
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
            Self::H => 1,
            Self::H2O => 0,
            Self::K => 1,
            Self::LAC => 0,
            Self::LDH => 0,
            Self::MSG => 0,
            Self::N2 => 0,
            Self::NAD => 1,
            Self::NADH => 0,
            Self::Na => 1,
            Self::NaCl => 0,
            Self::O2 => 0,
            Self::PFK => 0,
            Self::PGK => 0,
            Self::PYR => 0,

            // Amino Acids
            // https://www.ncbi.nlm.nih.gov/pmc/articles/PMC1450267
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

            // Starch
            Self::AML(_) => 0,
            Self::AMN(_) => 0,
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
            Self::H => MolarMass::from_gpmol(1.00794),
            Self::H2O => MolarMass::from_gpmol(18.0153),
            Self::K => MolarMass::from_gpmol(39.0983),
            Self::LAC => MolarMass::from_gpmol(89.07),
            Self::LDH => MolarMass::from_kgpmol(144.0),
            Self::MSG => MolarMass::from_gpmol(169.11),
            Self::N2 => MolarMass::from_gpmol(28.0134),
            Self::NAD => MolarMass::from_gpmol(663.43),
            Self::NADH => MolarMass::from_gpmol(665.125),
            Self::Na => MolarMass::from_gpmol(22.989769),
            Self::NaCl => MolarMass::from_gpmol(58.44),
            Self::O2 => MolarMass::from_gpmol(31.9988),
            Self::PFK => MolarMass::from_kgpmol(85.0),
            Self::PGK => MolarMass::from_kgpmol(45.0),
            Self::PYR => MolarMass::from_gpmol(88.06),

            // Amino Acids
            // https://www.ncbi.nlm.nih.gov/pmc/articles/PMC3302019/table/tab1/
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
            Self::THR => MolarMass::from_gpmol(119.1),
            Self::TRP => MolarMass::from_gpmol(204.2),
            Self::TYR => MolarMass::from_gpmol(181.2),
            Self::VAL => MolarMass::from_gpmol(117.5),

            // Starch
            Self::AML(len) => Self::GLC.molar_mass()*f64::from(*len),
            Self::AMN(len) => Self::GLC.molar_mass()*f64::from(*len),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Substance;

    #[test]
    fn test_fmt() {
        let ca = Substance::Ca;
        let na = Substance::Na;
        let cl = Substance::Cl;
        let salt = Substance::NaCl;
        assert_eq!(format!("{}", ca), "Calcium (Ca 2+)");
        assert_eq!(format!("{:?}", ca), "Ca");
        assert_eq!(format!("{}", na), "Sodium (Na+)");
        assert_eq!(format!("{:?}", na), "Na");
        assert_eq!(format!("{}", cl), "Chloride (Cl-)");
        assert_eq!(format!("{:?}", cl), "Cl");
        assert_eq!(format!("{}", salt), "Salt (NaCl)");
        assert_eq!(format!("{:?}", salt), "NaCl");
    }
}
