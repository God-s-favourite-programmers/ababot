import datetime
import logging
import pytz
import discord
from discord.ext import commands, tasks

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)


class Cleaning(commands.Cog):
    """
    Lets moderators bulk delete messages
    """

    def __init__(self, client):
        """Save the channel named bot-commands."""

        self.client = client
        self.name = type(self).__name__
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(
            self.client.get_all_channels(), guild=self.guild, name='bot-commands').id
        self.channel = self.client.get_channel(self.channelId)

        logger.info(
            f"Deploying cleanup to Channel: {self.channelId}")
        self.cleanup.start()

    # Commands
    @commands.command()
    @commands.has_role("Los Jefes")
    async def clear(self, ctx, amount=10):
        """Delete [amount] of messages."""
        await ctx.channel.purge(limit=amount)

    @clear.error
    async def clear_error(self, ctx, error):
        """If error is due to lack of permission, notify the user of their lack of permission. Otherwise warn of error."""

        if isinstance(error, commands.errors.CheckFailure):
            await ctx.reply("You don't have permisson to use that command")
        else:
            logger.error(error)
            await ctx.send(f"An error ocurred while clearing: {error}")

    # Loops
    @tasks.loop(hours=1)
    async def cleanup(self):
        """At midnight delete all messages in saved channel."""
        if 2 > datetime.datetime.now(tz=local_timezone).hour >= 0:

            while len(await self.channel.history(limit=123).flatten()) > 0:
                await self.channel.purge(limit=100)

    @cleanup.error
    async def cleanup_error(self, error):
        logger.error(error)
        await self.channel.send(f"An error ocurred in {self.name}: {error}")


def setup(client):
    """Sets up the cog."""

    client.add_cog(Cleaning(client))
