# General
from src.cogs.abakus.event import Event
from src.cogs.abakus.async_helpers import post, remind, check_message, clean
from src.cogs.abakus.event_parser import get_event, list_events
from discord.ext import commands, tasks
import discord
import asyncio
import datetime
import logging
import pytz
import traceback
import time

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
        self.channelId = discord.utils.get(self.client.get_all_channels(), guild=self.guild, name='abakus').id
        self.channel = self.client.get_channel(self.channelId)

        logger.info(f"Deploying reminder and poster to Channel: {self.channelId}")
        
        self.poster.start()
        self.poster_cleanup.start()
        self.reminder.start()
        self.reminder_cleanup.start()


    @commands.command()
    async def is_running(self, ctx):
        """Report on the state of the loops."""

        logger.info(
            f"Poster running: {self.poster.is_running()} | Reminder running: {self.reminder.is_running()} | Poster Cleanup running: {self.poster_cleanup.is_running()} | Reminder Cleanup running: {self.reminder_cleanup.is_running()}")
        await ctx.send(f"Poster running: \t\t {self.poster.is_running()}\nReminder running:\t{self.reminder.is_running()}\nPoster Cleanup running: \t\t {self.poster_cleanup.is_running()}\nReminder Cleanup running:\t{self.reminder_cleanup.is_running()}")


    @commands.command()
    @commands.has_role("Los Jefes")
    async def restart_abakus(self, ctx):
        """Restart both loops."""

        logger.info("Restarting loops")
        async with ctx.typing():
            self.poster.cancel()
            self.poster_cleanup.cancel()
            self.reminder.cancel()
            self.reminder_cleanup.cancel()
            await asyncio.sleep(2)
            self.poster.start()
            self.reminder.start()
            self.poster_cleanup.start()
            self.reminder_cleanup.start()

        if (self.poster.is_running() and self.reminder.is_running() and self.reminder_cleanup.is_running() and self.poster_cleanup.is_running()):
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

        dev_event: Event = Event(f"Dev event{time.time()}", "This is a dummy event for dev purposes", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(hours=2), "Discord", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(minutes=11), "https://abakus.no/events/2901","https://thumbor.abakus.no/40uu7jE2T02LcSrnDonxbJGSmd0=/0x500/Skjermbilde2021-05-1_sRKgopY.png")

        await post(self.channel, dev_event, self.client)

    @commands.command()
    @commands.has_role("Los Jefes")
    async def post_updated(self, ctx):
        """Post dev event.
        
        A dev event is an event starting in two hours, with registration opening in 11 minutes."""

        dev_event: Event = Event(f"Dev eent{time.time()}", "Updated dummyevent", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(hours=2), "Discord", datetime.datetime.now(tz=local_timezone)+datetime.timedelta(minutes=11), "https://abakus.no/events/2901","https://thumbor.abakus.no/40uu7jE2T02LcSrnDonxbJGSmd0=/0x500/Skjermbilde2021-05-1_sRKgopY.png")

        await post(self.channel, dev_event, self.client)

    @tasks.loop(minutes=10)
    async def poster(self):
        """Retrieve a list of all events and post them."""

        logger.info("Poster started")
        events: list[Event] = [y for y in [get_event(x) for x in list_events()] if y != None]

        for event_object in events:
            await post(self.channel, event_object, self.client)


    @tasks.loop(minutes=1)
    async def reminder(self):
        """Retreive the message history of the saved channel and check each message."""

        logger.info("Reminder started")


        async for message in self.channel.history(limit=123):
            if len(message.embeds) > 0:
                ok, users, msg = await check_message(message, self.delta)
                if not ok:
                    continue
                for user in users:
                    await remind(user, msg, self.client)

    @tasks.loop(minutes=10)
    async def poster_cleanup(self):
        """Delete postings for which the event is older than two days"""

        logger.info("Poster Cleanup started")

        others = False

        async for message in self.channel.history(limit=123):
            if message.author == self.client.user:
                await clean(message, others)

    @tasks.loop(minutes=10)
    async def reminder_cleanup(self):
        """Delete postings for which the event is older than two days"""

        logger.info("Reminder Cleanup started")

        others = True
        async for member in self.guild.fetch_members(limit=150):
            if not member.bot:
                if not member.dm_channel:
                    await member.create_dm()
                async for message in member.dm_channel.history(limit=123):
                    if message.author == self.client.user:
                        await clean(message, others)


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
