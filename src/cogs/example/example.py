import logging
import discord
from discord.ext import commands

logger = logging.getLogger(__name__)

# Example cog class


class Example(commands.Cog):

    def __init__(self, client):
        """Save the refrence to the client."""

        self.client = client
        self.name = type(self).__name__
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

    # Events
    @commands.Cog.listener()
    async def on_ready(self):
        pass

    # Commands
    @commands.command()
    async def ping(self, ctx):
        """Responds with Pong!"""
        await ctx.send("Pong!")

    @ping.error
    async def ping_error(self, ctx, error):
        """Handles error originating from ping command"""
        logger.error(error)
        print(f"Error ocurred: {error}")
        await ctx.send(f"An error ocurred: {error}")


def setup(client):
    """Sets up the cog."""

    client.add_cog(Example(client))
