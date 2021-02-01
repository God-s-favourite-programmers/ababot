import os
import datetime
import asyncio
import discord
from discord.ext import commands
import eventParser
from helperFunctions import get_event_properties, generate_message, get_dm_history

client = commands.Bot(command_prefix = "!")


@client.event
async def on_ready():
    print("Bot is ready")
    for guild in client.guilds:
        channelId = discord.utils.get(client.get_all_channels(), guild=guild, name='ababot').id
        client.loop.create_task(poster(channelId))
        client.loop.create_task(reminder(channelId))


async def poster(channelId):
    template="eventTemplate.txt"
    await client.wait_until_ready()
    channel = client.get_channel(channelId)
    while True:
        messages = await channel.history(limit=123).flatten()
        messages = [x.content for x in messages]
        events = [eventParser.get_event(x) for x in eventParser.list_events()]
        for event in events:
            msg = generate_message(event, template)
            if msg not in messages:
                await channel.send(msg)
                await asyncio.sleep(5)


async def reminder(channelId):
    channel = client.get_channel(channelId)
    template = "reminderTemplate.txt"
    while True:
        messages = await channel.history(limit=123).flatten()
        for message in messages:
            if message.author == client.user:
                event = get_event_properties(message, template)
                if event["regsitrationOpen"] != "None":
                    signupTime = datetime.datetime.strptime(event["regsitrationOpen"], '%Y-%m-%d %H:%M:%S')
                    currentTime = datetime.datetime.now()
                    delta = datetime.timedelta(minutes=10)
                    if currentTime+delta >= signupTime:
                        msg = generate_message(event, template)
                        for reaction in message.reactions:
                            async for user in reaction.users():
                                alerts = get_dm_history(user)
                                if msg not in alerts:
                                    await user.send(msg)
                                    

if __name__ == "__main__":
    if os.path.isfile("/token/token.txt"):
        print("Found token.txt, attempting to use saved token")
        with open("/token/token.txt", "r") as f:
            token = f.read()
    else:
        print("token.txt file not found. Run the container with a volume, to avoid this problem in the future")
        token = input("Provide token manually: ")
        with open("/token/token.txt", "w") as f:
            f.write(token)
        print("Token written to token.txt, reuse the volume next time to avoid providing the token manually")
    

    client.run(token)
