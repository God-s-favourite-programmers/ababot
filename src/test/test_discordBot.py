import os
import time

def testBot():
    import sys, os
    myPath = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, myPath + '/../')
    import discordBot
    token = os.environ["TOKEN"]
    discordBot.client.run(token)
    time.sleep(5)
    discordBot.client.logout()