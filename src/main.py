import getToken
from discord.ext import commands
import discord
import asyncio
import os
import logging

logging.basicConfig(
    filename='/ababot/ababot.log',
    format="[%(asctime)s]: %(levelname)s - %(name)s - %(message)s",
    filemode='w',
    level=logging.INFO
)

logger = logging.getLogger(__name__)
intents = discord.Intents.default()
intents.members = True
client = commands.Bot(command_prefix=".", intents=intents, help_command=None)


@client.event
async def on_ready():
    """Indicate that the bot is ready."""

    print("AbaBot is ready")
    logger.info("Bot started")
    load_all_cogs()


def load_all_cogs():
    """Load all cogs found in src/cogs/ where the python file has the same name as its directory."""

    logger.info("Loading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs") if f.name != "example"]
    for cog in cogs:
        client.load_extension(f"src.cogs.{cog}.{cog}")


def unload_all_cogs():
    """Unload all cogs found in src/cogs/ where the oython file has the same name as its directory."""

    logger.info("Unloading cogs")
    cogs = [f.name for f in os.scandir("./src/cogs") if f.name != "example"]
    for cog in cogs:
        client.unload_extension(f"src.cogs.{cog}.{cog}")


@client.command()
@commands.has_role("Los Jefes")
async def reload(ctx):
    """Unload all cogs before loading all cogs."""

    logger.info("Reloading all cogs")
    async with ctx.typing():
        unload_all_cogs()
        load_all_cogs()
    await ctx.send("Reload complete")


@reload.error
async def reload_error(ctx, error):
    """If error is due to lack of permission, notify the user of their lack of permission. Otherwise warn of error."""

    if isinstance(error, commands.errors.CheckFailure):
        await ctx.reply("You don't have permisson to use that command")
    else:
        logger.error(error)
        await ctx.send(f"An error ocurred while reloading: {error}")


if __name__ == "__main__":
    logger.info("Running client")
    client.run(getToken.token)
