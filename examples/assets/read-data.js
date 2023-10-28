const fs = require('node:fs');
const data = fs.readFileSync("examples/assets/jq_data.json");
const items = JSON.parse(data);
const output = items.map((item) => item.commit.committer.name);

console.log(output)
