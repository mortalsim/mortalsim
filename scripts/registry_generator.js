
const yaml = require('yaml');
const fs = require('fs');
const path = require('path');

const layerPath = path.join(__dirname, '..', 'src', 'sim', 'layer');

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

