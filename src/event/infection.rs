use simple_si_units::mechanical::Frequency;

use crate::sim::Organism;

use super::{Event, NumType};

/// Characteristics of an infection
/// https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7150340/
#[derive(Debug, Clone, PartialEq)]
pub struct InfectionProperties<O: Organism> {
    // name of the infection
    name: &'static str,
    // Unique identifier for this specific infection
    id: String,
    // location of the infection
    location: O::AnatomyType,
    // The ability to enter and multiply in the host.
    infectivity: NumType,
    // The ability to produce a clinical reaction after infection occurs.
    pathogenicity: NumType,
    // The ability to produce a severe pathological reaction.
    virulence: NumType,
    // The ability to produce a poisonous reaction.
    toxicity: NumType,
    // The ability to penetrate and spread throughout the tissue.
    invasiveness: NumType,
    // The ability to stimulate an immunological response. 
    antigenicity: NumType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Infection<O: Organism> {
    Virus(InfectionProperties<O>),
    Bacteria(InfectionProperties<O>),
    Fungus(InfectionProperties<O>),
    Parasite(InfectionProperties<O>),
}

impl<O: Organism> Event for Infection<O> {}
