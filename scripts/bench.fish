# This script is has the following prerequisites (aside from fish):
# - hyperfine
# - dust (can be installed with "cargo install dust-lang")
# - jq
# - nodejs
# - nushell
# - dielectron.json (can be downloaded from https://opendata.cern.ch/record/304)

hyperfine
      --shell none
      --parameter-list data_path examples/assets/seaCreatures.json,examples/assets/jq_data.json,dielectron.json
      --warmup 3
      "dust -c '(length (from_json input))' -p {data_path}"
      "jq 'length' {data_path}"
      "node --eval \"require('node:fs').readFile('{data_path}', (err, data)=>{console.log(JSON.parse(data).length)})\""
      "nu -c 'open {data_path} | length'"