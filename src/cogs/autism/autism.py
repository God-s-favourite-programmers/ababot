import logging
import discord
from discord.ext import commands

logger = logging.getLogger(__name__)

# Example cog class


class Autism(commands.Cog):

    def __init__(self, client):
        """Save the refrence to the client."""
        self.client = client
        self.name = type(self).__name__
        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(
            self.client.get_all_channels(), guild=self.guild, name='Helpdesk').id
        self.channel = self.client.get_channel(self.channelId)
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

    # Events
    @commands.Cog.listener()
    async def on_ready(self):
        pass

    # Commands
    """@commands.command()
    async def ping(self, ctx):
        Responds with Pong!
        await ctx.send("Pong!")"""
j
    """@ping.error
    async def ping_error(self, ctx, error):
        Handles error originating from ping command
        logger.error(error)
        print(f"Error ocurred: {error}")
        await ctx.send(f"An error ocurred: {error}")"""
    
    @commands.command()

    async def em(self, ctx, name="Lounge"):
        async for mem in self.guild.fetch_members(limit=150):
            print(mem)
            if mem.voice != None:
                await mem.move_to(self.channel)


def setup(client):
    """Sets up the cog."""

    client.add_cog(Autism(client))
