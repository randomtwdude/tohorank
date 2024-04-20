# Python script to generate the character list file
# used by tohorank to generate its data file
# This script parses the dataset file of tohosort.

import re
import sys

if len(sys.argv) <= 1:
    print("Usage: generate_charas.py [tohosort data.js]")
    sys.exit()
else:
    file_name = sys.argv[1]

with open(file_name, 'r') as data, open('touhous.txt', 'w') as chara:
    # start
    chara.write("# This is the character list used by Tohorank to generate its data.\n")
    line = data.readline()
    date = re.search(r'dataSetVersion = "([^"]*)"', line)
    chara.write("# Generated from tohosort's dataset: " + date.group(1) + "\n")
    # skip to the character part
    while line := data.readline():
        if re.search(r'dataSet\[dataSetVersion\].characterData', line):
            break
    # parse!
    while line := data.readline():
        if match := re.search(r'name: "([^"]*)"', line):
            # name
            chara.write("\n" + match.group(1) + ";")
        elif match := re.search(r'series: \[.*\]', line):
            # works tag
            works = re.findall(r'"([^"]*?)"', line)
            for work in works:
                chara.write(" " + work)
        elif match := re.search(r'stage: \[.*\]', line):
            # stages tag
            stages = re.findall(r'"([^"]*?)"', line)
            for stage in stages:
                chara.write(" " + stage)
        if re.search(r'pc98: true', line):
            chara.write("; pc98")
        if re.search(r'nameless: true', line):
            chara.write("; nameless")
        if re.search(r'notgirl: true', line):
            chara.write("; notgirl")