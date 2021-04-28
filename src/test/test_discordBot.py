import asyncio
import sys, os

def testBot():
    try:
        myPath = os.path.dirname(os.path.abspath(__file__))
        sys.path.insert(0, myPath + '/../')
        from discordBot import client
        args = sys.argv
        token = args[1]
        
        
        async def go():
            try:
                await asyncio.wait_for(client.start(token), timeout=25)
            except asyncio.TimeoutError:
                pass

        loop = asyncio.get_event_loop()
        loop.run_until_complete(go())
        if client.is_running():
            client.close()
            return 0
        else:
            return 1
    except:
        return 1
testBot()