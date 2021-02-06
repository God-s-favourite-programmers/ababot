import os
import datetime
import logging
import asyncio
import discord
from discord.ext import commands
import getToken

logging.basicConfig(filename='/ababot/ababot.log', format="[%(asctime)s]: %(name)s - %(levelname)s - %(message)s", filemode='w', level=logging.INFO)
logger = logging.getLogger(__name__)
client = commands.Bot(command_prefix="!")

@client.event
async def on_ready():
    print("AbaBot is ready")
    logging.info("Bot started")

if __name__ == "__main__":
    logging.info("Loading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs")]
    for cog in cogs:
        #if os.path.isfile("./src/cogs/{cog}/{cog}.py"):
        client.load_extension(f"src.cogs.{cog}.{cog}")
    logging.info("Running client")
    client.run(getToken.token)
