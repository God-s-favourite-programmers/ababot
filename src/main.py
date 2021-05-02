import logging
logging.basicConfig(
    filename='/ababot/ababot.log',
    format="[%(asctime)s]: %(levelname)s - %(name)s - %(message)s",
    filemode='w',
    level=logging.INFO
    )
    
import os
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


@client.command()
@commands.has_role("Los Jefes")
async def reload(ctx):
    logger.info("Reloading all cogs")
    async with ctx.typing():
        unload_all_cogs()
        load_all_cogs()
    await ctx.send("Reload complete")

@reload.error
async def reload_error(ctx, error):
    if isinstance(error, commands.errors.CheckFailure):
        await ctx.send("You don't have permisson to use that command")
    else:
        logger.error(error)
        await ctx.send(f"An error ocurred while reloading: {error}")


if __name__ == "__main__":
    load_all_cogs()
    logger.info("Running client")
    client.run(getToken.token)
