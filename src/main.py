from json import load
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

@client.command()
async def reload(ctx):
    logger.info("Reloading all cogs")
    async with ctx.typing():
        unload_all_cogs()
        load_all_cogs()
    await ctx.send("Reload complete")

def load_all_cogs():
    logger.info("Loading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs")]
    for cog in cogs:
        client.load_extension(f"src.cogs.{cog}.{cog}")

def unload_all_cogs():
    logger.info("Unloading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs")]
    for cog in cogs:
        client.unload_extension(f"src.cogs.{cog}.{cog}")



if __name__ == "__main__":
    load_all_cogs()
    logger.info("Running client")
    client.run(getToken.token)
