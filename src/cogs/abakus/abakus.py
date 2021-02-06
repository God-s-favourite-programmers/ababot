# General
import datetime
import logging
import asyncio
# Discord
import discord
from discord.ext import commands, tasks

# Custom
from src.cogs.abakus import eventParser
from src.cogs.abakus.helperFunctions import get_event_properties, generate_message, get_dm_history

logger = logging.getLogger(__name__)

class Abakus(commands.Cog):

    def __init__(self, client):
        self.client = client
        self.name = type(self).__name__
        print(f"Cog {self.name} loaded")
        logging.info(f"Cog {self.name} loaded")

    @commands.Cog.listener()
    async def on_ready(self):
        self.guild = self.client.guilds[0]
        self.channelId = discord.utils.get(self.client.get_all_channels(), guild=self.guild, name='ababot').id
        self.channel = self.client.get_channel(self.channelId)

        self.poster.start()
        self.reminder.start()

    @tasks.loop(minutes=10)
    async def poster(self):
        logging.info("Poster started")
        template = "eventTemplate.txt"
        messages = await self.channel.history(limit=123).flatten()
        messages = [x.content for x in messages]
        events = [eventParser.get_event(x) for x in eventParser.list_events()]
        for event in events:
            if (event["registrationOpen"] != None):
                msg = generate_message(event, template)
                if msg not in messages:
                    await self.channel.send(msg)
                    logging.debug("Event listed")
                    await asyncio.sleep(5)

    @tasks.loop(minutes=1)
    async def reminder(self):
        logging.info("Reminder started")
        template = "reminderTemplate.txt"
        regexTemplate = "eventRegexPattern.txt"
        messages = await self.channel.history(limit=123).flatten()
        for message in messages:
            if message.author == self.client.user:
                event = get_event_properties(message, regexTemplate)
                if event["registrationOpen"] != "None":
                    signupTime = event["registrationOpen"]
                    currentTime = datetime.datetime.now()
                    delta = datetime.timedelta(minutes=10)
                    if currentTime+delta >= signupTime:
                        msg = generate_message(event, template)
                        print(msg)
                        for reaction in message.reactions:
                            async for user in reaction.users():
                                alerts = await get_dm_history(user)
                                if msg not in alerts:
                                    logging.debug("Direct message sent")
                                    await user.send(msg)

    async def cog_command_error(self, ctx, error):
        print("Big fucking error")
        await self.channel.send("Abakus cog has stopped working")
        self.client.close()

def setup(client):
    client.add_cog(Abakus(client))
