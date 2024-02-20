#!/usr/bin/fish
# This script is has the following prerequisites (aside from fish):
# - hyperfine
# - dust (can be installed with "cargo install dust-lang")
# - jq
# - nodejs
# - nushell
# - dielectron.json (can be downloaded from https://opendata.cern.ch/record/304)

hyperfine \
      --shell none \
      --parameter-list data_path examples/assets/seaCreatures.json \
      --warmup 3 \
      "dust -c 'length(json:parse(fs:read_file(\"{data_path}\")))'" \
      "jq 'length' {data_path}" \
      "node --eval \"require('node:fs').readFile('{data_path}', (err, data)=>{console.log(JSON.parse(data).length)})\"" \
      "nu -c 'open {data_path} | length'"

hyperfine \
      --shell none \
      --parameter-list data_path examples/assets/jq_data.json \
      --warmup 3 \
      "dust -c 'length(json:parse(fs:read_file(\"{data_path}\")))'" \
      "jq 'length' {data_path}" \
      "node --eval \"require('node:fs').readFile('{data_path}', (err, data)=>{console.log(JSON.parse(data).length)})\"" \
      "nu -c 'open {data_path} | length'"

hyperfine \
      --shell none \
      --parameter-list data_path dielectron.json \
      --warmup 3 \
      "dust -c 'length(json:parse(fs:read_file(\"{data_path}\")))'" \
      "jq 'length' {data_path}" \
      "node --eval \"require('node:fs').readFile('{data_path}', (err, data)=>{console.log(JSON.parse(data).length)})\"" \
      "nu -c 'open {data_path} | length'"
