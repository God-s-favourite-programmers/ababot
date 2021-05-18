# General
from discord.ext import commands, tasks
from discord import channel
import discord
import asyncio
import datetime
import logging
import pytz
import json

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)


class Soup(commands.Cog):
    """
    Uploads helpful files to #suppekjøkkenet
    """

    def __init__(self, client):
        """Save the channel named suppekjøkkenet."""

        self.client = client
        self.name = type(self).__name__
        self.delta = datetime.timedelta(minutes=10)
        with open("src/cogs/soup/codes.json", "r", encoding="utf") as f:
            self.codes = json.load(f)
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(
            self.client.get_all_channels(), guild=self.guild, name='suppekjøkkenet').id
        self.channel = self.client.get_channel(self.channelId)

    @commands.command()
    async def kok(self, ctx: commands.Context, code: str, info: str, extra_info: str = ""):
        """Upload kok.
        
        Formats a new message based on the provided information and post it to the saved channel."""

        message: discord.Message = ctx.message
        attachments = message.attachments
        name: str = self.codes[code]

        if len(attachments) < 1:
            await ctx.send("Did you forget to attach a file?")
            return
        elif len(attachments) > 1:
            await ctx.send("Oh no, I can't handle all those files at once")
            return

        kok: str = f"**{code}** : {name}\n{info}\n{extra_info}"
        await self.channel.send(kok, file=await attachments[0].to_file())

    @kok.error
    async def kok_error(self, ctx, error):
        """If the file is too large report on it, otherwise reply with ambiguous error."""

        logger.error(error)
        error_as_str: str = str(error)
        if error_as_str == "Command raised an exception: HTTPException: 413 Payload Too Large (error code: 40005): Request entity too large":
            await ctx.reply("Uh oh, your soup is simply too large, I can't carry this")
        else:
            await ctx.reply("I don't understand. Did you maybe forget to close you quotes?")


def setup(client):
    """Sets up the cog."""

    client.add_cog(Soup(client))
