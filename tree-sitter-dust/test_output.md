| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `dust -c '(length (from_json input))' -p seaCreatures.json` | 2.9 ± 0.4 | 2.5 | 6.7 | 1.00 |
| `jq 'length' seaCreatures.json` | 36.7 ± 4.4 | 34.7 | 65.0 | 12.59 ± 2.14 |
| `node --eval "require('node:fs').readFile('seaCreatures.json', (err, data)=>{console.log(JSON.parse(data).length)})"` | 241.2 ± 13.3 | 227.7 | 273.2 | 82.63 ± 11.00 |
| `nu -c 'open seaCreatures.json \| length'` | 54.0 ± 3.3 | 50.3 | 69.2 | 18.49 ± 2.51 |
| `dust -c '(length (from_json input))' -p jq_data.json` | 7.9 ± 0.8 | 6.6 | 12.5 | 2.70 ± 0.43 |
| `jq 'length' jq_data.json` | 44.8 ± 0.6 | 43.5 | 47.3 | 15.36 ± 1.87 |
| `node --eval "require('node:fs').readFile('jq_data.json', (err, data)=>{console.log(JSON.parse(data).length)})"` | 245.2 ± 7.1 | 235.4 | 259.7 | 84.00 ± 10.46 |
| `nu -c 'open jq_data.json \| length'` | 65.9 ± 5.0 | 62.0 | 90.5 | 22.57 ± 3.22 |
| `dust -c '(length (from_json input))' -p dielectron.json` | 1079.5 ± 22.7 | 1043.8 | 1121.5 | 369.86 ± 45.46 |
| `jq 'length' dielectron.json` | 1365.0 ± 20.3 | 1318.5 | 1400.1 | 467.67 ± 57.07 |
| `node --eval "require('node:fs').readFile('dielectron.json', (err, data)=>{console.log(JSON.parse(data).length)})"` | 1910.8 ± 47.9 | 1855.9 | 1985.7 | 654.66 ± 80.97 |
| `nu -c 'open dielectron.json \| length'` | 2001.2 ± 65.1 | 1923.2 | 2112.7 | 685.65 ± 85.98 |
