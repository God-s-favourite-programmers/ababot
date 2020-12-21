import os
import json
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
            


@client.command(aliases=['begin'])
async def start(ctx):
    guild = ctx.message.guild
    channel = discord.utils.get(client.get_all_channels(), guild=guild, name='ababot').id
    client.loop.create_task(poster(channel))

    
async def reminder(channelId):
    await client.wait_until_ready()
    channel = client.get_channel(channelId)
    while True:
        messages = await channel.history(limit=123).flatten()
        for message in messages:
            #Use regex to fill values
            startTime = 0
            signupTime = 0
            eventName = ""
            currentTime = datetime.datetime.now()
            delta = datetime.timedelta(minutes=10)
            if currentTime+delta == signupTime:
                pass
                #For user that has reacted
                    #Message user


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