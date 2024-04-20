# Python script to generate the groups.rs file
# This script parses the dataset file of tohosort.

import re
import sys

if len(sys.argv) <= 1:
    print("Usage: generate_groups.py [tohosort data.js]")
    sys.exit()
else:
    file_name = sys.argv[1]

with open(file_name, 'r') as data, open('groups.rs', 'w') as output:
    # PART 0 - preamble
    output.write("// Character groups for detailed stats\n")
    line = data.readline()
    date = re.search(r'dataSetVersion = "([^"]*)"', line)
    output.write("// Generated from tohosort's dataset: " + date.group(1) + "\n")
    output.write("\n")
    output.write("use serde::{Serialize, Deserialize};\n")
    output.write("use std::str::FromStr;\n")
    output.write("use strum_macros::EnumIter;\n")
    output.write("\n")
    output.write("// Tell the compiler to stop complaining\n")
    output.write("#[allow(non_camel_case_types)]\n")
    output.write("\n")
    output.write("// Group by the work they appeared in\n")
    output.write("// taken from tohosort\n")
    output.write("\n")

    # PART 1 - enum tags
    output.write("#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Debug, Clone, EnumIter)]\n")
    output.write("pub enum Tags {\n")
    ## start parsin'
    while line := data.readline():
        match = re.search(r'Filter by Series Entry', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    ## format: { name: "*NAME*", tooltip: "*TITLE*", key: "*KEY*" },
    ##     --> *KEY*,
    start = False
    while line := data.readline():
        match = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for tag in match:
                output.write("\t" + tag + ",\n")
        elif start:
            break
    output.write("\t// Stages\n")
    ## stage tags
    while line := data.readline():
        match = re.search(r'Filter by Stage Enemy Appearances', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    start = False
    while line := data.readline():
        match = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for stage in match:
                output.write("\t" + stage + ",\n")
        elif start:
            break

    output.write("}\n\n")

    # PART 2 - enum -> group name
    ## preamble
    output.write("impl Tags {\n")
    output.write("\t// Names of groups (touhou works and stages)\n")
    output.write("\tpub fn name(&self) -> &'static str {\n")
    output.write("\t\tmatch *self {\n")
    ## parse!
    data.seek(0)
    while line := data.readline():
        match = re.search(r'Filter by Series Entry', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    ## format: { name: "*NAME*", tooltip: "*TITLE*", key: "*KEY*" },
    ##     --> Tag::*KEY* => "*NAME*"
    start = False
    while line := data.readline():
        match = re.findall(r'name: ("[^"]*").*?key: "(.*)"', line)
        if match:
            start = True
            for (name, key) in match:
                output.write("\t\t\tTags::" + key + "\t=> " + name + ",\n")
        elif start:
            break
    ## stage
    output.write("\t\t\t// Stages\n")
    while line := data.readline():
        match = re.search(r'Filter by Stage Enemy Appearances', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break

    start = False
    while line := data.readline():
        match = re.findall(r'name: ("[^"]*").*?key: "([^"]*)"', line)
        if match:
            start = True
            for (name, tag) in match:
                output.write("\t\t\tTags::" + tag + "\t=> " + name + ",\n")
        elif start:
            break
    output.write("\t\t}\n\t}\n")

    # PART 3 - enum -> touhou titles
    ## preamble
    output.write("\t// Touhou game titles.\n")
    output.write("\tpub fn exname(&self) -> &'static str {\n")
    output.write("\t\tmatch *self {\n")
    ## parse!
    data.seek(0)
    while line := data.readline():
        match = re.search(r'Filter by Series Entry', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    ## format: { name: "*NAME*", tooltip: "*TITLE*", key: "*KEY*" },
    ##     --> Tag::*KEY* => "*NAME*"
    start = False
    while line := data.readline():
        match = re.findall(r'tooltip: ("[^"]*").*?key: "([^"]*)"', line)
        match2 = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for (title, key) in match:
                output.write("\t\t\tTags::" + key + "\t=> " + title + ",\n")
        elif match2:
            # "book" doesn't have a tooltip
            start = True
            for key in match2:
                output.write("\t\t\tTags::" + key + "\t=> \"\",\n")
        elif start:
            break
    ## stages have no titles, but we need to include them still
    output.write("\t\t\t// Stages\n")
    while line := data.readline():
        match = re.search(r'Filter by Stage Enemy Appearances', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break

    start = False
    while line := data.readline():
        match = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for tag in match:
                output.write("\t\t\tTags::" + tag + "\t=> \"\",\n")
        elif start:
            break

    output.write("\t\t}\n\t}\n")

    # PART 4 - separate series and stages
    output.write("\t// Returns true if it's a series tag\n")
    output.write("\tpub fn is_series_tag(&self) -> bool {\n")
    output.write("\t\tmatch self {\n")
    data.seek(0)
    while line := data.readline():
        match = re.search(r'Filter by Stage Enemy Appearances', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break

    start = False
    while line := data.readline():
        match = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for tag in match:
                output.write("\t\t\tTags::" + tag + "\t=> false,\n")
        elif start:
            break

    output.write("\t\t\t_\t=> true,\n\t\t}\n\t}\n}\n")

    # PART 5 - string -> enum
    output.write("\n// String to name utility\n")
    output.write("impl FromStr for Tags {\n")
    output.write("\ttype Err = ();\n")
    output.write("\tfn from_str(input: &str) -> Result<Tags, Self::Err> {\n")
    output.write("\t\tmatch input.to_lowercase().as_str() {\n")
    ## start parsin'
    data.seek(0)
    while line := data.readline():
        match = re.search(r'Filter by Series Entry', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    ## format: { name: "*NAME*", tooltip: "*TITLE*", key: "*KEY*" },
    ##     --> "*KEY*" => Ok(Tags::*KEY*),
    start = False
    while line := data.readline():
        match = re.findall(r'tooltip: "([^ ]*).*key: "([^"]*)"', line)
        match2 = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for (number, tag) in match:
                output.write("\t\t\t\"" + tag.lower() + "\"\t| \"th" + number + "\"\t=> Ok(Tags::" + tag + "),\n")
        elif match2:
            # "book" doesn't have a tooltip
            start = True
            for tag in match2:
                output.write("\t\t\t\"" + tag.lower() + "\"\t=> Ok(Tags::" + tag + "),\n")
        elif start:
            break
    output.write("\t\t\t// Stages\n")
    ## stage tags
    while line := data.readline():
        match = re.search(r'Filter by Stage Enemy Appearances', line)
        if match:
            data.readline() # the next line is key for the option so we skip that
            break
    start = False
    while line := data.readline():
        match = re.findall(r'key: "([^"]*)"', line)
        if match:
            start = True
            for stage in match:
                output.write("\t\t\t\"" + stage.lower() + "\"\t=> Ok(Tags::" + stage + "),\n")
        elif start:
            break
    output.write("\t\t\t_\t=> Err(()),\n")

    output.write("\t\t}\n\t}\n}")