
const yaml = require('yaml');
const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, '..', 'config', 'substances.yaml');
const substancePath = path.join(__dirname, '..', 'mortalsim-core', 'src', 'substance', 'substance.rs')

const substanceConfigs = yaml.parse(fs.readFileSync(configPath, 'utf8'));

function stringifyCharge(charge) {
    if(charge == 1) return '+';
    else if(charge > 1) return `${charge}+`;
    else if(charge == -1) return '-';
    else if(charge < -1) return `${charge}-`;
    return ''
}

function fmtValue(num) {
    if(Number.isInteger(num)) return num.toFixed(1);
    return num;
}

fs.writeFileSync(substancePath, `
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
${Object.entries(substanceConfigs).map(([sid, cfg]) =>
`    /// ${cfg.name} (${sid}${stringifyCharge(cfg.charge)})
    ${sid},
`).join('')}
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
${Object.entries(substanceConfigs).map(([sid, cfg]) =>
`            Self::${sid} => "${cfg.name}",
`).join('')}
        }
    }

    /// Overall substance charge
    pub fn charge(&self) -> i8 {
        match self {
${Object.entries(substanceConfigs).map(([sid, cfg]) =>
`            Self::${sid} => ${cfg.charge},
`).join('')}
        }
    }

    /// Typical molar mass of the substance
    pub fn molar_mass(&self) -> MolarMass<f64> {
        match self {
${Object.entries(substanceConfigs).map(([sid, cfg]) =>
`            Self::${sid} => MolarMass::from_gpmol(${fmtValue(cfg.molar_mass)}),
`).join('')}
        }
    }

    /// Typical density of the substance
    pub fn density(&self) -> Density<f64> {
        match self {
${Object.entries(substanceConfigs).map(([sid, cfg]) =>
`            Self::${sid} => Density::from_gpcc(${fmtValue(cfg.density)}),
`).join('')}
        }
    }

    /// Typical molar volume of the substance at body temperature
    pub fn molar_volume(&self) -> crate::units::chemical::MolarVolume<f64> {
        self.molar_mass() / self.density()
    }
}
`);
