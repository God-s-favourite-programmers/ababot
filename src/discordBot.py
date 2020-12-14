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



if __name__ == "__main__":
    if os.path.isfile("token.txt"):
        with open("token.txt", "r") as f:
            token = f.read()
        print(token)
        client.run(token)
    else:
        with open("token.txt", "w") as f:
            f.write("TOKEN")
        print("Replace TOKEN in token.txt file")