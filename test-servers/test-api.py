#!/usr/bin/env python3
import requests
import time
import os
import sys

# simulate a session stop if -k is passed
do_kill = '-k' in sys.argv

# start a new dns enumeration session
api_server = 'http://localhost:8666'
args = ["dns", "--target", "something.com", "--payloads", "data/1000.txt"]

resp = requests.post(f'{api_server}/api/session/new', json=args)
if not resp.ok:
    print(f"ERROR: {resp}")
    quit()

session_id = resp.json()

print(f"session {session_id} started ...")

# list sessions
print(requests.get(f'{api_server}/api/sessions').json())

num = 0

while True:
    time.sleep(1)
    # get session status
    resp = requests.get(f'{api_server}/api/session/{session_id}')
    if resp.ok:
        os.system("clear")
        session = resp.json()
        print(f"started_at={session['started_at']}\n")
        print('\n'.join(session['output']))
        print()
        print(session['statistics'])

        if 'completed' in session and session['completed'] is not None:
            break

        if do_kill and num >= 2:
            print("killing ...")
            # stop the session
            resp = requests.get(f'{api_server}/api/session/{session_id}/stop')
            print(resp.text)

    num += 1

print("\n")
print(session)
