import os
import json
import datetime
import re
import asyncio
import discord
from discord.ext import commands
import eventParser

client = commands.Bot(command_prefix = "!")

@client.event
async def on_ready():
    print("Bot is ready")


async def poster(channelId):
    await client.wait_until_ready()
    channel = client.get_channel(channelId)
    while True:
        messages = await channel.history(limit=123).flatten()
        messages = [x.content for x in messages]
        events = eventParser.listEvents()
        events = [eventParser.getEvent(x) for x in events]
        for event in events:
            msg = f"""> **{event['name']}**\n{event['description']}\nBegins on {event['eventTime']} in {event['eventLocation']}\nRegistrations begin on {event['regsitrationOpen']}\n{event['url']}"""
            if msg not in messages:
                await channel.send(msg)
                await asyncio.sleep(5)
            else:
                events.remove(event)

    
async def reminder(channelId):
    await client.wait_until_ready()
    channel = client.get_channel(channelId)

    #region Send dummy event
    currentTime = datetime.datetime.now()
    delta = datetime.timedelta(minutes=11)
    signupTime = currentTime+delta
    startTime = signupTime+delta
    eventName = "Test event"
    eventDescription = "Description of event"
    eventLocation = "Discord"
    url = "https://github.com/Areskiko/ababot"
    msg = f"""> **{eventName}**\n{eventDescription}\nBegins on {startTime} in {eventLocation}\nRegistrations begin on {datetime.datetime.strftime(signupTime, '%Y-%m-%d %H:%M:%S')}\n{url}"""
    #await channel.send(msg)
    #endregion
    
    while True:
        messages = await channel.history(limit=123).flatten()
        for message in messages:
            if message.author == client.user:
                print("\n\n\n-------------------\n")
                pattern = """> \*\*(.*?)\*\*
(.*?)
Begins on (.*?) in (.*?)
Registrations begin on (.*?)
(.*?)"""
                messageSearch = re.search(pattern, message.content)
                #Use regex to fill values
                startTime = messageSearch.group(3)
                signupTime = messageSearch.group(5)
                if signupTime == "None":
                    pass
                else:
                    eventName = messageSearch.group(1)
                    print(eventName)
                    print(signupTime)
                    signupTime = datetime.datetime.strptime(signupTime, '%Y-%m-%d %H:%M:%S')
                    currentTime = datetime.datetime.now()
                    delta = datetime.timedelta(minutes=10)
                    print(currentTime+delta)
                    print(signupTime)
                    if currentTime+delta >= signupTime:
                        msg = f"""The event {eventName} opens its registration in less than ten minutes at {signupTime}
The event itself starts at {startTime}"""
                        for reaction in message.reactions:
                            async for user in reaction.users():
                                if user.dm_channel:
                                    pass
                                else:
                                    await user.create_dm()
                                alerts = await user.dm_channel.history(limit=123).flatten()
                                alerts = [x.content for x in alerts]
                                if msg not in alerts:
                                    await user.send(msg)

@client.command(aliases=['begin'])
async def start(ctx):
    guild = ctx.message.guild
    channel = discord.utils.get(client.get_all_channels(), guild=guild, name='ababot').id
    client.loop.create_task(poster(channel))
    client.loop.create_task(reminder(channel))

if __name__ == "__main__":
    if os.path.isfile("token.txt"):
        print("Found token.txt, attempting to use saved token")
        with open("token.txt", "r") as f:
            token = f.read()
    else:
        print("token.txt file not found. Run the container with a volume, to avoid this problem in the future")
        token = input("Provide token manually: ")
        with open("token.txt", "w") as f:
            f.write(token)
        print("Token written to token.txt, reuse the volume next time to avoid providing the token manually")
    
    client.run(token)