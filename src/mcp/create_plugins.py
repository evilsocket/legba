#!/usr/bin/env python3
import requests
import re
import os

plugin_list = requests.get('https://raw.githubusercontent.com/wiki/evilsocket/legba/_Sidebar.md').text
in_plugins_section = False 
regex = r"^\* \[(.+)\]\(([^)]+)\)(.*)$"
data = ''

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
        desc = desc.strip().lstrip("(").rstrip(")").strip()
        # print(f'{name}: {link} - {desc}')
        if desc == '':
            data += f'# {name}\n\n'
        else:
            data += f'# {name} ({desc})\n\n'

        link = link.replace('https://github.com/', 'https://raw.githubusercontent.com/wiki/' ).replace('/legba/wiki/', '/legba/') + ".md"
        print(f'collecting {link} ...')
        data += requests.get(link).text + "\n\n"

script_folder = os.path.dirname(os.path.abspath(__file__))
prompt_folder = os.path.join(script_folder, 'plugins.prompt')

with open(prompt_folder, 'w') as f:
    f.write(data)
