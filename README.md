# vdftool

vdftool is a single binary containing a set of tools for manipulating VDF[^1] files.
At the moment it only offers commands for converting to/from JSON.

[^1]: Valve Data Format, aka KeyValues

## Usage

```
# write some sample data to a file
$ cat << EOF > ./data.vdf
"AppState"
{
  "appid" "440"
  "name" "Team Fortress 2"
  "installdir" "Team Fortress 2"
}
EOF

# `-t` preserves the top-level key
# `-p` pretty-prints the JSON output
$ vdftool vdf2json -tp ./data.vdf

{
  "AppState": {
    "appid": 440,
    "installdir": "Team Fortress 2",
    "name": "Team Fortress 2"
  }
}

# transform data using jq
$ vdftool vdf2json -tp ./data.vdf \
  | jq 'map_values({appid: (.appid + 200), name})' \
  | vdftool json2vdf

"AppState"
{
  "appid" "640"
  "name" "Team Fortress 2"
}
```
