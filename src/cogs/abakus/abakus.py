# General
from src.cogs.abakus.event import event
from src.cogs.abakus.helper_functions import get_event_properties, generate_message, get_dm_history
from src.cogs.abakus.event_parser import get_event, list_events
from discord.ext import commands, tasks
from discord import channel
import discord
import asyncio
import datetime
import logging
from discord.ext.commands.core import is_owner
import pytz

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)


class Abakus(commands.Cog):

    def __init__(self, client):
        """Save the channel named ababot and start both loops."""

        self.client = client
        self.name = type(self).__name__
        self.delta = datetime.timedelta(minutes=10)
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(
            self.client.get_all_channels(), guild=self.guild, name='ababot').id
        self.channel = self.client.get_channel(self.channelId)

        logger.info(
            f"Deploying reminder and poster to Channel: {self.channelId}")
        self.poster.start()
        self.reminder.start()

    @commands.command()
    async def is_running(self, ctx):
        """Report on the state of the loops."""

        logger.info(
            f"Poster running: {self.poster.is_running()} | Reminder running: {self.reminder.is_running()}")
        await ctx.send(f"Poster running: \t\t {self.poster.is_running()}\nReminder running:\t{self.reminder.is_running()}")

    @commands.command()
    @commands.has_role("Los Jefes")
    async def restart_abakus(self, ctx):
        """Restart both loops."""

        logger.info("Restarting loops")
        try:
            async with ctx.typing():
                self.poster.cancel()
                self.reminder.cancel()
                await asyncio.sleep(5)
                self.poster.start()
                self.reminder.start()
        except Exception as e:
            raise e

        if (self.poster.is_running() and self.reminder.is_running()):
            logger.info("All loops are running")
            await ctx.send("Restart complete")
        else:
            logger.warning("Not all loops are running")
            await ctx.send("Not all loops are running") 

    @restart_abakus.error
    async def restart_error(self, ctx, error):
        """Report on restart error."""

        logger.error(error)
        await ctx.send(f"An error ocurred while restarting: {error}")

    async def post(self, event_object: event) -> None:
        """Post an event in the saved channel if the exact same post does not allready exist."""

        template = "eventTemplate.txt"
        msg = generate_message(event_object, template)

        if msg == None:
            raise ValueError("Message is none")

        messages = [x.content for x in await self.channel.history(limit=123).flatten()]

        if msg not in messages and len(msg) > 0:
            await self.channel.send(msg)
            logger.debug(f"Event {event.get_name} listed")

    @commands.command()
    @commands.has_role("Los Jefes")
    async def post_dev_test(self, ctx):
        """Post dev event.
        
        A dev event is an event starting in two hours, with registration opening in 11 minutes."""

        dev_event: event = event("Dev event",
                                 "This is a dummy event for dev purposes",
                                 datetime.datetime.now(
                                     tz=local_timezone)+datetime.timedelta(hours=2),
                                 "Discord",
                                 datetime.datetime.now(
                                     tz=local_timezone)+datetime.timedelta(minutes=11),
                                 "N/A")
        await self.post(dev_event)

    @post_dev_test.error
    async def post_dev_test_error(self, ctx, error):
        """If error is due to lack of permission, notify the user of their lack of permission. Otherwise warn of error."""

        if isinstance(error, commands.errors.CheckFailure):
            await ctx.reply("You don't have permission to use that command")
        else:
            logger.error(error)
            await ctx.send(f"An error ocurred: {error}")

    @tasks.loop(minutes=10)
    async def poster(self):
        """Retrieve a list of all events and post them."""

        try:
            logger.info("Poster started")
            events: list[event] = [y for y in [
                get_event(x) for x in list_events()] if y != None]

            for event_object in events:
                await self.post(event_object)

        except ConnectionError as e:
            pass

        except Exception as e:
            logger.debug(f"Poster caught error: {e}")
            raise e

    async def remind(self, user: discord.User, msg: str) -> None:
        """Send a message to a user if the exact same message does not allready exist."""

        alerts = await get_dm_history(user)

        if msg not in alerts and len(msg) > 0:
            logger.debug("Direct message sent")
            await user.send(msg)

    async def check_message(self, message: discord.Message) -> None:
        """Retreive the information of an event posting and check if the signup time is within the wanted timedelta."""

        template = "reminderTemplate.txt"
        regexTemplate = "eventRegexPattern.txt"
        event_object = get_event_properties(message, regexTemplate)

        if event_object == None:
            return

        signupTime = event_object.get_registration_open()
        currentTime = datetime.datetime.now(tz=local_timezone)

        if currentTime+self.delta >= signupTime:
            msg = generate_message(event_object, template)

            for reaction in message.reactions:
                async for user in reaction.users():
                    await self.remind(user, msg)

    @tasks.loop(minutes=1)
    async def reminder(self):
        """Retreive the message history of the saved channel and check each message."""
        try:
            logger.info("Reminder started")
            messages = [x for x in await self.channel.history(limit=123).flatten() if x.author == self.client.user]
            for message in messages:
                await self.check_message(message)
        except Exception as e:
            logger.debug(f"Reminder caught error: {e}")
            raise e

    @reminder.error
    @poster.error
    async def cog_command_error(self, error):
        """Report on any errors."""

        print(f"Abakus cog error: {error}")
        await self.channel.send(f"An error ocurred in {self.name}: {error}")
        logger.error(error)


def setup(client):
    """Sets up the cog."""

    client.add_cog(Abakus(client))
