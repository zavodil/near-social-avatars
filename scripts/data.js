import {Avataaars} from "./avatars.js";
import fs from 'fs';

const USER_ID="zavodil.near";
const APP_NAME="avtr";
const CONTRACT_ID="social.near"; // v1.social08.testnet

let data = {};
let types = Object.keys(Avataaars.paths).filter((p)=>{return Object.keys(Avataaars.paths[p]).length > 1});

types.map(type => {
    data[type] = {};

    Object.keys(Avataaars.paths[type]).map(item => {
        let value = Avataaars.paths[type][item](`%${type.toUpperCase()}_COLOR_1%`, `%${type.toUpperCase()}_COLOR_2%`);

        if(type === "clothingGraphic"){
            value = `<g transform="translate(77, 60)">${value.replaceAll('fill=\"#fff\"', 'fill=\"%CLOTHING_GRAPHICS_COLOR%\"')}</g>`
        }
        if(type === "clothing" && item === "graphicShirt"){
            value = value.replace('<g transform=\"translate(77, 60)\">%CLOTHING_COLOR_2%</g>', "%CLOTHING_GRAPHICS%")
        }

        data[type][item] = {
            name: item.replace(/([A-Z])/g, ' $1').trim(),
            src: value
                .replace(/\n/g, '')
                .replace(/\s{2,}/g,' ')
                .trim(),
            price: "0"
        };


    });
})

Object.keys(data).map(type => {
    let result = {
        data: {
            [USER_ID]: {
                [APP_NAME]: {
                    components: {
                        [type]: data[type]
                    }
                }
            }
        }
    };

    fs.writeFile(`./data/set_data_${type}.json`, `near call ${CONTRACT_ID} set '${JSON.stringify(result)}' --accountId $USER_ID --deposit 0.5 --gas 300000000000000`, function (err,data) {
        if (err) {
            return console.log(err);
        }
        console.log(type, result);
    });

    //Object.keys(data[type]).map(item => {
        //console.log(JSON.parse(data[type][item]))
        //console.log(data[type][item])
    //});
});


// export color categories
Object.entries(Avataaars.colors).forEach(type => {
    let colorsCategories = {};
    Object.entries(type[1]).forEach(color => {
        if(!colorsCategories.hasOwnProperty(type[0])){
            colorsCategories[type[0]] = {};
        }
        colorsCategories[type[0]] =  Object.assign( colorsCategories[type[0]], {
            [color[0]]: ""
        });
    });

    console.log(colorsCategories)

    let result = {
        data: {
            [USER_ID]: {
                [APP_NAME]: {
                    colors_categories: colorsCategories
                }
            }
        }
    };

    fs.writeFile(`./data/set_data_colors_categories_${type[0]}.json`, `near call ${CONTRACT_ID} set '${JSON.stringify(result)}' --accountId $USER_ID --deposit 0.2 --gas 300000000000000`, function (err,data) {
        if (err) {
            return console.log(err);
        }
        console.log(result);
    });
});

// export colors

Object.entries(Avataaars.colors).forEach(type => {
    let colors = {};
    Object.entries(type[1]).forEach(color => {
        //console.log(color)
        /*colors[color[0]] = {
            name: color[0].replace(/([A-Z])/g, ' $1').trim(),
            src: color[1]

        }*/

        /*
        colors = Object.assign(colors, {
            name: color[0].replace(/([A-Z])/g, ' $1').trim(),
            src: color[1]
        });

         */

        colors[color[0]] = {
            name: color[0].replace(/([A-Z])/g, ' $1').trim(),
            src: color[1]

        }
    });

    console.log(colors)

    let result = {
        data: {
            [USER_ID]: {
                [APP_NAME]: {
                    colors
                }
            }
        }
    };

    fs.writeFile(`./data/set_data_${type[0]}.json`, `near call ${CONTRACT_ID} set '${JSON.stringify(result)}' --accountId $USER_ID --deposit 0.2 --gas 300000000000000`, function (err,data) {
        if (err) {
            return console.log(err);
        }
        console.log(result);
    });
});


