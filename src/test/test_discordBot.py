import asyncio
import sys, os

def testBot():
    myPath = os.path.dirname(os.path.abspath(__file__))
    sys.path.insert(0, myPath + '/../')
    from discordBot import client
    token = os.environ["TOKEN"]
    
    
    async def go():
        try:
            await asyncio.wait_for(client.start(token), timeout=5)
        except asyncio.TimeoutError:
            await client.close()

    loop = asyncio.get_event_loop()
    loop.run_until_complete(go())