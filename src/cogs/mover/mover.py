import logging
import discord
from discord.errors import HTTPException
from discord.ext import commands
from discord.ext.commands.errors import CommandInvokeError

logger = logging.getLogger(__name__)

# Example cog class


class Mover(commands.Cog):
    """
    Moves users connected to VC
    """

    def __init__(self, client):
        """Save the refrence to the client."""
        self.client = client
        self.name = type(self).__name__
        self.guild = self.client.guilds[0]
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

    # Events

    @commands.command()

    async def em(self, ctx, name="Lounge"):
        """Move users to [Voice Channel].

        Defaults to Lounge.
        """
        try:
            self.channelId = discord.utils.get(
                self.client.get_all_channels(), guild=self.guild, name=name).id
            self.channel = self.client.get_channel(self.channelId)
            async for mem in self.guild.fetch_members(limit=150):
                print(mem)
                if mem.voice != None:
                    await mem.move_to(self.channel)
        except (CommandInvokeError, AttributeError,HTTPException):   
            await ctx.send("Please specify channel correctly")


def setup(client):
    """Sets up the cog."""
    client.add_cog(Mover(client))
