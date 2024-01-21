use std::fmt;
use crate::units::chemical::MolarMass;

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
            Substance::ADP => "Adenosine Diphosphate",
            Substance::AMP => "Adenosine Monophosphate",
            Substance::ATP => "Adenosine Triphosphate",
            Substance::Ca => "Calcium",
            Substance::CO2 => "Carbon Dioxide",
            Substance::Cl => "Chloride",
            Substance::GLC => "Alpha D Glucose",
            Substance::GLCL => "L-Glucose",
            Substance::H => "Hydrogen",
            Substance::H2O => "Water",
            Substance::K => "Potassium",
            Substance::LAC => "Lactate",
            Substance::LDH => "Lactate Dehydrogenase",
            Substance::MSG => "Monosodium Glutamate",
            Substance::N2 => "Dinitrogen",
            Substance::NAD => "Nicotinamide Adenine Dinucleotide",
            Substance::NADH => "Reduced Nicotinamide Adenine Dinucleotide",
            Substance::Na => "Sodium",
            Substance::NaCl => "Salt",
            Substance::O2 => "Dioxygen",
            Substance::PFK => "Phosphofructokinase",
            Substance::PGK => "Phosphoglycerate Kinase",
            Substance::PYR => "Pyruvate",
        }
    }
    /// Overall substance charge
    pub fn charge(&self) -> i8 {
        match self {
            Substance::ADP => 0,
            Substance::AMP => 0,
            Substance::ATP => 0,
            Substance::Ca => 2,
            Substance::CO2 => 0,
            Substance::Cl => -1,
            Substance::GLC => 0,
            Substance::GLCL => 0,
            Substance::H => 1,
            Substance::H2O => 0,
            Substance::K => 1,
            Substance::LAC => 0,
            Substance::LDH => 0,
            Substance::MSG => 0,
            Substance::N2 => 0,
            Substance::NAD => 1,
            Substance::NADH => 0,
            Substance::Na => 1,
            Substance::NaCl => 0,
            Substance::O2 => 0,
            Substance::PFK => 0,
            Substance::PGK => 0,
            Substance::PYR => 0,
        }
    }
    /// Typical molar mass of the substance
    pub fn molar_mass(&self) -> MolarMass<f64> {
        match self {
            Substance::ADP => MolarMass::from_gpmol(427.201),
            Substance::AMP => MolarMass::from_gpmol(347.2212),
            Substance::ATP => MolarMass::from_gpmol(507.18),
            Substance::Ca => MolarMass::from_gpmol(40.078),
            Substance::CO2 => MolarMass::from_gpmol(44.01),
            Substance::Cl => MolarMass::from_gpmol(35.453),
            Substance::GLC => MolarMass::from_gpmol(180.156),
            Substance::GLCL => MolarMass::from_gpmol(180.156),
            Substance::H => MolarMass::from_gpmol(1.00794),
            Substance::H2O => MolarMass::from_gpmol(18.0153),
            Substance::K => MolarMass::from_gpmol(39.0983),
            Substance::LAC => MolarMass::from_gpmol(89.07),
            Substance::LDH => MolarMass::from_kgpmol(144.0),
            Substance::MSG => MolarMass::from_gpmol(169.11),
            Substance::N2 => MolarMass::from_gpmol(28.0134),
            Substance::NAD => MolarMass::from_gpmol(663.43),
            Substance::NADH => MolarMass::from_gpmol(665.125),
            Substance::Na => MolarMass::from_gpmol(22.989769),
            Substance::NaCl => MolarMass::from_gpmol(58.44),
            Substance::O2 => MolarMass::from_gpmol(31.9988),
            Substance::PFK => MolarMass::from_kgpmol(85.0),
            Substance::PGK => MolarMass::from_kgpmol(45.0),
            Substance::PYR => MolarMass::from_gpmol(88.06),
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
