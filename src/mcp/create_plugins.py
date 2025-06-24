#!/usr/bin/env python3
import requests
import re
import os

plugin_list = requests.get('https://raw.githubusercontent.com/wiki/evilsocket/legba/_Sidebar.md').text
in_plugins_section = False 
regex = r"^\* \[(.+)\]\(([^)]+)\)(.*)$"
data = '## Available plugins identifiers\n\n'

for line in plugin_list.split('\n'):
    line = line.strip()
    if line == '* Plugins':
        in_plugins_section = True
        continue
    elif not in_plugins_section:
        continue

    match = re.match(regex, line)
    if match:
        name, link, desc = match.groups()
        ident = link.split('/')[-1].split('.')[0]
        data += f'* {ident}\n'

data += "\nUse the plugin_info tool with the plugin identifier as it is to get information about a plugin, its options and how to use it."

script_folder = os.path.dirname(os.path.abspath(__file__))
prompt_folder = os.path.join(script_folder, 'plugins.prompt')

with open(prompt_folder, 'w') as f:
    f.write(data)
