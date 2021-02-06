import logging
import discord
from discord.ext import commands

logger = logging.getLogger(__name__)

# Example cog class
class Example(commands.Cog):

    def __init__(self, client):
        self.client = client
        self.name = type(self).__name__
        print(f"Cog {self.name} loaded")
        logging.info(f"Cog {self.name} loaded")
    

    # Events
    @commands.Cog.listener()
    async def on_ready(self):
        pass


    # Commands
    @commands.command()
    async def ping(self, ctx):
        await ctx.send("Pong!")

    async def cog_command_error(self, ctx, error):
        print("Big fucking error occured")

def setup(client):
    client.add_cog(Example(client))