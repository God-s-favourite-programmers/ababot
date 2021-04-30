# General
import asyncio
import datetime
import logging
from discord.ext.commands.core import is_owner
import pytz
local_timezone = pytz.timezone("Europe/Oslo")
# Discord
import discord
from discord import channel
from discord.ext import commands, tasks

# Custom
from src.cogs.abakus.event_parser import get_event, list_events
from src.cogs.abakus.helper_functions import get_event_properties, generate_message, get_dm_history
from src.cogs.abakus.event import event

logger = logging.getLogger(__name__)

class Abakus(commands.Cog):

    def __init__(self, client):
        self.client = client
        self.name = type(self).__name__
        self.delta = datetime.timedelta(minutes=10)
        print(f"Cog {self.name} loaded")
        logger.info(f"Cog {self.name} loaded")

    @commands.Cog.listener()
    async def on_ready(self):
        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(self.client.get_all_channels(), guild=self.guild, name='ababot').id
        self.channel = self.client.get_channel(self.channelId)
        logger.info(f"Deploying reminder and poster to Channel: {self.channelId}")
        self.poster.start()
        self.reminder.start()

    @commands.command()
    async def is_running(self, ctx):
        logger.info(f"Poster running: {self.poster.is_running()} | Reminder running: {self.reminder.is_running()}")        
        await ctx.send(f"Poster running: \t\t {self.poster.is_running()}\nReminder running:\t{self.reminder.is_running()}")

    @commands.command()
    async def restart(self, ctx):
        logger.info("Restarting loops")
        try:
            async with ctx.typing():
                self.poster.start()
                self.reminder.start()
        except Exception as e:
            raise e
        if (self.poster.is_running() and self.reminder.is_running()):
            logger.info("All loops are running")
            await ctx.send("Restart complete")
        else:
            logger.warning("Npt all loops are running")
            await ctx.send("Not all loops are running")

    @restart.error
    async def restart_error(self, ctx, error):
        logger.error(error)
        ctx.send(f"An error ocurred while reloading: {error}")

    async def post(self, event_object:event) -> None:
        template = "eventTemplate.txt"
        msg = generate_message(event_object, template)
        if msg == None:
            raise ValueError("Message is none")
        messages = [x.content for x in await self.channel.history(limit=123).flatten()]
        if msg not in messages and len(msg) > 0:
            await self.channel.send(msg)
            logger.debug(f"Event {event.get_name} listed")

    @commands.command()
    async def post_dev_test(self, ctx):
        dev_event:event = event("Dev event",
            "This is a dummy event for dev purposes",
            datetime.datetime.now(tz=local_timezone)+datetime.timedelta(hours=2),
            "Discord",
            datetime.datetime.now(tz=local_timezone)+datetime.timedelta(minutes=11),
            "N/A")
        await self.post(dev_event)        

    @tasks.loop(minutes=10)
    async def poster(self):
        try:
            logger.info("Poster started")
            events:list[event] = [y for y in [get_event(x) for x in list_events()] if y != None]
            for event_object in events:
                await self.post(event_object)
            
        except ConnectionError as e:
            pass

        except Exception as e:
            logger.debug(f"Poster caught error: {e}")
            raise e




    async def remind(self, user:discord.User, msg:str) -> None:
        alerts = await get_dm_history(user)
        if msg not in alerts and len(msg) > 0:
            logger.debug("Direct message sent")
            await user.send(msg)

    async def check_message(self, message:discord.Message) -> None:
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
        print(f"Abakus cog error: {error}")
        await self.channel.send(f"Abakus cog has stopped working due to: {error}")
        logger.error(error)

def setup(client):
    client.add_cog(Abakus(client))
