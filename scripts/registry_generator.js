
const yaml = require('yaml');
const fs = require('fs');
const path = require('path');

const rootPath = path.join(__dirname, '..');
const layerPath = path.join(rootPath, 'src', 'sim', 'layer');
const targetPath = path.join(rootPath, 'src', 'sim', 'component', 'registry.rs');

Object.defineProperty(String.prototype, 'cap', {
  value: function() {
    return this.charAt(0).toUpperCase() + this.slice(1);
  },
  enumerable: false
});

// Start with core to make sure it's always the first one
const layerList = ["core"];

fs.readdirSync(layerPath)
    .filter(f => fs.statSync(path.join(layerPath, f)).isDirectory())
    .forEach(dir => {
        console.log(dir);
        if(!layerList.includes(dir)) {
            layerList.push(dir);
        }
    });

function combos(list) {
     var fn = function(active, rest, a) {
        if (!active.length && !rest.length)
            return;
        if (!rest.length) {
            // Don't include the full list
            if(active.length < layerList.length) {
                a.push(active);
            }
        } else {
            fn([...active, ...rest.slice(0, 1)], rest.slice(1), a);
            fn(active, rest.slice(1), a);
        }
        return a;
    }
    return fn([], list, []);
}

const layerCombos = combos(layerList);

const layerMap = {};

layerList.forEach(layer => {
    layerMap[layer] = [];
});

layerCombos.forEach(clayers => {
    layerList.forEach(layer => {
        if(clayers.includes(layer)) {
            layerMap[layer].push(clayers);
        }
    })
});

console.log(layerCombos);
console.log(layerMap);

function getWrapperName(list) {
    return `${list.map(l => l.cap()).join('')}Wrapper`
}

function layersToBounds(list) {
    return `${list.map(l => `${l.cap()}Component<O>`).join(' + ')}`;
}

function layerImpl(wrapperName, impls, noimpls) {
    return `
${impls.map(impl => `
impl<O: Organism + 'static, T: ${layersToBounds(impls)}> ${impl.cap()}Component<O> for ${wrapperName}<O, T> {
    fn ${impl}_init(&mut self, initializer: &mut ${impl.cap()}Initializer<O>) {
        self.0.${impl}_init(initializer)
    }
    fn ${impl}_connector(&mut self) -> &mut ${impl.cap()}Connector<O> {
        self.0.${impl}_connector()
    }
}
`).join('')}

${noimpls.map(impl => `
impl<O: Organism + 'static, T: ${layersToBounds(impls)}> ${impl.cap()}Component<O> for ${wrapperName}<O, T> {
    fn ${impl}_init(&mut self, _initializer: &mut ${impl.cap()}Initializer<O>) {
        panic!("Improper wrapper method called!")
    }
    fn ${impl}_connector(&mut self) -> &mut ${impl.cap()}Connector<O> {
        panic!("Improper wrapper method called!")
    }
}
`).join('')}
`
}

    fs.writeFileSync(targetPath, `
/*
 * THIS FILE IS AUTOMATICALLY GENERATED.
 * SOURCE: scripts/registry_generator.js
 */

use std::marker::PhantomData;
use crate::sim::organism::Organism;
use crate::sim::layer::{
${layerList.map(l =>
`    ${l.cap()}Component,
    ${l.cap()}Initializer,
    ${l.cap()}Connector`).join(',\n')},
};
use super::SimComponent;

pub trait ComponentWrapper<O: Organism>: SimComponent<O> + ${layerList.map(l => `${l.cap()}Component<O>`).join(' + ')} {}
${layerCombos.map(items => {
    let wrapperName = getWrapperName(items);
    return `
pub struct ${wrapperName}<O: Organism, T: ${layersToBounds(items)} + 'static>(pub T, pub PhantomData<O>);

impl<O: Organism + 'static, T: ${layersToBounds(items)}> SimComponent<O> for ${wrapperName}<O, T> {
    fn id(&self) -> &'static str {
        self.0.id()
    }
    fn attach(self, registry: &mut ComponentRegistry<O>) {
        self.0.attach(registry)
    }
    fn run(&mut self) {
        self.0.run();
    }
}
${layerImpl(wrapperName, items, layerList.filter(l => !items.includes(l)))}
impl<O: Organism + 'static, T: ${layersToBounds(items)}> ComponentWrapper<O> for ${wrapperName}<O,T> {}
`
}).join('')}

pub struct ComponentRegistry<O: Organism> (Vec<Box<dyn ComponentWrapper<O>>>);

impl<O: Organism + 'static> ComponentRegistry<O> {
    pub fn add_component(&mut self, component: impl SimComponent<O>) {
        component.attach(self)
    }
${layerCombos.map(items=> `
    pub fn add_${items.join('_')}_component(&mut self, component: impl ${layersToBounds(items)} + 'static) {
        self.0.push(Box::new(${getWrapperName(items)}(component, PhantomData)))
    }
`).join('')}
}
`)