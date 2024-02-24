
const yaml = require('yaml');
const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, '..', 'config', 'substances.yaml');
const substancePath = path.join(__dirname, '..', 'src', 'substance', 'substance.rs')

const substanceConfigs = yaml.parse(fs.readFileSync(configPath, 'utf8'));

function stringifyCharge(charge) {
    if(charge == 1) return '+';
    else if(charge > 1) return `${charge}+`;
    else if(charge == -1) return '-';
    else if(charge < -1) return `${charge}-`;
    return ''
}

function fmtMass(num) {
    if(Number.isInteger(num)) return num.toFixed(1);
    return num;
}

fs.writeFileSync(substancePath, `
/*
 * THIS FILE IS AUTOMATICALLY GENERATED.
 * SOURCE: scripts/substance_writer.js
 */

use crate::units::chemical::MolarMass;
use std::fmt;

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
`            Self::${sid} => MolarMass::from_gpmol(${fmtMass(cfg.molar_mass)}),
`).join('')}
        }
    }
}
`);
