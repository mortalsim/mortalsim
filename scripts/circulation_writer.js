const yaml = require('yaml');
const fs = require('fs');
const path = require('path');

const configPath = path.join(__dirname, '..', 'config');

fs.readdirSync(configPath)
    .filter(f => fs.statSync(path.join(configPath, f)).isDirectory())
    .forEach(dir => {
        console.log(dir);
        let configFile = fs.readFileSync(path.join(configPath, dir, 'closed_circulation.yaml'), 'utf8');
        let config = yaml.parse(configFile);
        writeCircFile(dir, config);
    });

function writeCircFile(namespace, config) {
    let namespaceCapitalized = namespace.charAt(0).toUpperCase() + namespace.slice(1);
    let vesselEnum = `${namespaceCapitalized}BloodVessel`;
    let anatomyEnum = `${namespaceCapitalized}AnatomicalRegion`
    let arteries = {};
    let veins = {};
    let bridges = {};
    let maxArterial = 0;
    let maxVenous = 0;

    function processArtery(entry, upstream, depth) {
        let artery = {
            id: entry.id,
            regions: entry.regions,
            upstream: upstream ? [upstream.id] : [],
            downstream: [...(entry.links || []).map(e => e.id), ...(entry.bridges || []).map(e => e)]
        }
        if(entry.bridges) {``
            entry.bridges.forEach(b => {
                if(!bridges[b]) bridges[b] = [];
                bridges[b].push(entry.id);
            })
        }
        arteries[entry.id] = artery;
        if(depth > maxArterial) {
            maxArterial = depth;
        }
        (entry.links || []).forEach(e => processArtery(e, artery, depth+1));
    }

    function processVein(entry, downstream, depth) {
        let vein = {
            id: entry.id,
            regions: entry.regions,
            downstream: downstream ? [downstream.id] : [],
            upstream: [...(entry.links || []).map(e => e.id), ...(bridges[entry.id] || [])]
        }
        veins[entry.id] = vein;
        if(depth > maxVenous) {
            maxVenous = depth;
        }
        (entry.links || []).forEach(e => processVein(e, vein, depth+1));
    }

    config.arterial.forEach(e => processArtery(e, null, 0));
    config.venous.forEach(e => processVein(e, null, 0));

    let allVessels = [...Object.values(arteries), ...Object.values(veins)];

    fs.writeFileSync(path.join(configPath, '..', config.path), `
/*
 * THIS FILE IS AUTOMATICALLY GENERATED.
 * SOURCE: config/${namespace}/circulation.yaml
 */
use std::collections::{HashMap, HashSet};

use crate::sim::layer::closed_circulation::{
    BloodVesselType, BloodVessel, VesselIter, AnatomicalRegionIter
};

use super::${namespaceCapitalized}AnatomicalRegion;

#[derive(Debug, Display, Hash, Clone, Copy, PartialEq, Eq, EnumString, IntoStaticStr)]
pub enum ${vesselEnum} {
    ${Object.keys(arteries).join(',\n    ')},
    ${Object.keys(veins).join(',\n    ')}
}

lazy_static! {
    static ref START_VESSELS: HashSet<${vesselEnum}> = {
        let mut vessel_list = HashSet::new();
        ${Object.values(arteries)
            .filter(a => a.upstream.length == 0)
            .map(a => `vessel_list.insert(${vesselEnum}::${a.id});`)
            .join('\n        ')}
        vessel_list
    };

    static ref ARTERIES: HashSet<${vesselEnum}> = {
        let mut vessel_list = HashSet::new();
        ${Object.values(arteries)
            .map(a => `vessel_list.insert(${vesselEnum}::${a.id});`)
            .join('\n        ')}
        vessel_list
    };

    static ref VEINS: HashSet<${vesselEnum}> = {
        let mut vessel_list = HashSet::new();
        ${Object.values(veins)
            .map(a => `vessel_list.insert(${vesselEnum}::${a.id});`)
            .join('\n        ')}
        vessel_list
    };

    static ref PRE_CAPILLARIES: HashSet<${vesselEnum}> = {
        let mut vessel_list = HashSet::new();
        ${Object.keys(bridges)
            .map(v => `vessel_list.insert(${vesselEnum}::${v});`)
            .join('\n        ')}
        vessel_list
    };

    static ref POST_CAPILLARIES: HashSet<${vesselEnum}> = {
        let mut vessel_list = HashSet::new();
        ${Object.values(bridges)
            .flat()
            .map(v => `vessel_list.insert(${vesselEnum}::${v});`)
            .join('\n        ')}
        vessel_list
    };
}

lazy_static! {
${allVessels.map(v => `
    static ref ${v.id.toUpperCase()}_UPSTREAM: HashSet<${vesselEnum}> = {
        ${v.upstream.length > 0 ? `let mut vessel_list = HashSet::new();
        ${v.upstream.map(x => `vessel_list.insert(${vesselEnum}::${x});`).join('\n        ')}
        vessel_list
        ` :
        `HashSet::new()`}
    };
`).join('')}
}

lazy_static! {
${allVessels.filter(v => v.downstream).map(v => `
    static ref ${v.id.toUpperCase()}_DOWNSTREAM: HashSet<${vesselEnum}> = {
        ${v.downstream.length > 0 ? `let mut vessel_list = HashSet::new();
        ${v.downstream.map(x => `vessel_list.insert(${vesselEnum}::${x});`).join('\n        ')}
        vessel_list
        ` :
        `HashSet::new()`}
    };
`).join('')}
}

lazy_static! {
${allVessels.map(v => `
    static ref ${v.id.toUpperCase()}_REGIONS: HashSet<${anatomyEnum}> = {
        let mut region_list = HashSet::new();
        ${v.regions.map(x => `region_list.insert(${anatomyEnum}::${x});`).join('\n        ')}
        region_list
    };
`).join('')}
}

impl BloodVessel for ${vesselEnum} {
    type AnatomyType = ${anatomyEnum};

    fn start_vessels<'a>() -> VesselIter<'a, Self> {
        VesselIter(START_VESSELS.iter())
    }
    fn arteries<'a>() -> VesselIter<'a, Self> {
        VesselIter(ARTERIES.iter())
    }
    fn veins<'a>() -> VesselIter<'a, Self> {
        VesselIter(VEINS.iter())
    }
    fn pre_capillaries<'a>() -> VesselIter<'a, Self> {
        VesselIter(PRE_CAPILLARIES.iter())
    }
    fn post_capillaries<'a>() -> VesselIter<'a, Self> {
        VesselIter(POST_CAPILLARIES.iter())
    }
    fn max_arterial_depth() -> u32 {${maxArterial}}
    fn max_venous_depth() -> u32 {${maxVenous}}
    fn max_cycle() -> u32 {${maxArterial + maxVenous}}
    fn vessel_type(&self) -> BloodVesselType {
        match self {
            ${Object.keys(arteries)
                .map(a => `${vesselEnum}::${a}`)
                .join(' |\n            ')} => BloodVesselType::Artery,

            ${Object.keys(veins)
                .map(v => `${vesselEnum}::${v}`)
                .join(' |\n            ')} => BloodVesselType::Vein,
        }
    }
    fn upstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            ${allVessels.map(v => `
            ${vesselEnum}::${v.id} => VesselIter(${v.id.toUpperCase()}_UPSTREAM.iter())`
            )}
        }
    }
    fn downstream<'a>(&self) -> VesselIter<'a, Self> {
        match self {
            ${allVessels.map(v => `
            ${vesselEnum}::${v.id} => VesselIter(${v.id.toUpperCase()}_DOWNSTREAM.iter())`
            )}
        }
    }
    fn regions<'a>(&self) -> AnatomicalRegionIter<Self::AnatomyType> {
        match self {
            ${allVessels.map(v => `
            ${vesselEnum}::${v.id} => AnatomicalRegionIter(${v.id.toUpperCase()}_REGIONS.iter())`
            )}
        }
    }
}
    `)
}
