# General
from src.cogs.abakus.event import Event
from src.cogs.abakus.async_helpers import post, remind, check_message
from src.cogs.abakus.event_parser import get_event, list_events
from discord.ext import commands, tasks
import discord
import asyncio
import datetime
import logging
import pytz
import traceback

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)


class Abakus(commands.Cog):
    """
    Reminds the server of events by abakus
    """

    def __init__(self, client):
        """Save the channel named ababot and start both loops."""

        self.client = client
        self.name = type(self).__name__
        self.delta = datetime.timedelta(minutes=10)

        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(self.client.get_all_channels(), guild=self.guild, name='ababot').id
        self.channel = self.client.get_channel(self.channelId)

        logger.info(f"Deploying reminder and poster to Channel: {self.channelId}")
        
        self.poster.start()
        self.reminder.start()


    @commands.command()
    async def is_running(self, ctx):
        """Report on the state of the loops."""

        logger.info(f"Poster running: {self.poster.is_running()} | Reminder running: {self.reminder.is_running()}")
        await ctx.send(f"Poster running: \t\t {self.poster.is_running()}\nReminder running:\t{self.reminder.is_running()}")


    @commands.command()
    @commands.has_role("Los Jefes")
    async def restart_abakus(self, ctx):
        """Restart both loops."""

        logger.info("Restarting loops")
        async with ctx.typing():
            self.poster.cancel()
            self.reminder.cancel()
            await asyncio.sleep(2)
            self.poster.start()
            self.reminder.start()

        if (self.poster.is_running() and self.reminder.is_running()):
            logger.info("All loops are running after restart")
            await ctx.send("Restart complete")
        else:
            logger.warning("Not all loops are running after restart")
            await ctx.send("Restart failed") 


    @commands.command()
    @commands.has_role("Los Jefes")
    async def post_dev_test(self, ctx):
        """Post dev event.
        
        A dev event is an event starting in two hours, with registration opening in 11 minutes."""

        dev_event: Event = Event("Dev event", "This is a dummy event for dev purposes", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(hours=2), "Discord", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(minutes=11), "N/A")

        await post(self.channel, dev_event)


    @tasks.loop(minutes=10)
    async def poster(self):
        """Retrieve a list of all events and post them."""

        logger.info("Poster started")
        events: list[Event] = [y for y in [get_event(x) for x in list_events()] if y != None]

        for event_object in events:
            await post(self.channel, event_object)


    @tasks.loop(minutes=1)
    async def reminder(self):
        """Retreive the message history of the saved channel and check each message."""

        logger.info("Reminder started")

        messages = [x for x in await self.channel.history(limit=123).flatten() if x.author == self.client.user]

        for message in messages:
            ok, users, msg = await check_message(message, self.delta)

            if not ok:
                continue
            for user in users:
                await remind(user, msg)


    @reminder.error
    @poster.error
    async def cog_task_error(self, error):
        """Report on any errors."""

        if not isinstance(error, ConnectionError):
            print(f"Abakus cog error: {error}")
            logger.error(error)

            await self.channel.send(f"An error ocurred: {error}\nTraceback:\n```{traceback.format_exc()}```")


    async def cog_command_error(self, ctx, error):
        """If error is due to lack of permission, notify the user of their lack of permission. Otherwise warn of error."""

        if isinstance(error, commands.errors.CheckFailure):
            await ctx.reply("You don't have permission to use that command")
        else:
            logger.error(error)
            await ctx.send(f"An error ocurred: {error}\nTraceback:\n```{traceback.format_exc()}```")



def setup(client):
    """Sets up the cog."""

    client.add_cog(Abakus(client))
