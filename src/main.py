import logging
logging.basicConfig(
    filename='/ababot/ababot.log',
    format="[%(asctime)s]: %(levelname)s - %(name)s - %(message)s",
    filemode='w',
    level=logging.INFO
    )
    
import os
import datetime
import asyncio
import discord
from discord.ext import commands
import getToken

logger = logging.getLogger(__name__)
client = commands.Bot(command_prefix="!")

@client.event
async def on_ready():
    print("AbaBot is ready")
    logger.info("Bot started")

if __name__ == "__main__":
    logger.info("Loading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs")]
    for cog in cogs:
        client.load_extension(f"src.cogs.{cog}.{cog}")
    logger.info("Running client")
    client.run(getToken.token)
