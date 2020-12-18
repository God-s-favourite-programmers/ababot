import os
import time

def testBot():
    import discordBot
    token = os.environ["TOKEN"]
    discordBot.client.run(token)
    time.sleep(5)
    discordBot.client.logout()