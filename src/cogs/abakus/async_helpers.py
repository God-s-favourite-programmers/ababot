import discord
import datetime
import logging
import pytz
from typing import Set, List

from src.cogs.abakus.helper_functions import get_event_properties, generate_message
from src.cogs.abakus.event import Event

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)

async def get_dm_history(user) -> List[discord.Message]:
    """Return a list of messages sent to the user"""
    if user.dm_channel:
        pass
    else:
        await user.create_dm()

    alerts = []
    async for alert in user.dm_channel.history(limit=123):
        if len(alert.embeds) > 0:
            alerts.append(alert)
    return alerts


async def check_message(message: discord.Message, delta) -> Set:
    """Retreive the information of an event posting and check if the signup time is within the wanted timedelta."""

    event_object:Event = get_event_properties(message)

    if event_object == None:
        return (False, None, None)

    signupTime:datetime.datetime = event_object.get_registration_open()
    currentTime:datetime = datetime.datetime.now(tz=local_timezone)

    if currentTime+delta >= signupTime:
        message_embed:discord.Embed = generate_message(event_object)
        users:list = []
        for reaction in message.reactions:
            users.extend(await reaction.users().flatten())

        return (True, users, message_embed)
    
    else:
        return (False, None, None)


async def remind(user: discord.User, message_embed: str, client) -> None:
    """Send a message to a user if the exact same message does not allready exist."""

    alerts:List[discord.Message] = await get_dm_history(user)
    alert_url:List[str] = [x.embeds[0].url for x in alerts]

    if message_embed.url not in alert_url:
        await user.send(embed=message_embed)

    else:
        for message in alerts:
            if len(message.embeds) > 0 and message.embeds[0].url == message_embed.url and message.author == client.user:
                await message.edit(embed=message_embed)


async def post(channel, event_object: Event, client) -> None:
    """Post an event in the saved channel or update if it currently exists."""

    message_embed:discord.Embed = generate_message(event_object)

    messages:List[discord.Message] = []
    messages_url:List[str] = []
    async for message in channel.history(limit=123):
        if len(message.embeds) > 0 and message.embeds[0].url not in messages_url:
            messages_url.append(message.embeds[0].url)
            messages.append(message)

    if message_embed.url not in messages_url:
        await channel.send(embed=message_embed)
        logger.debug(f"Event {event_object.get_name()} listed")

    else:
        for message in messages:
            if len(message.embeds) > 0 and message.embeds[0].url == message_embed.url and message.author == client.user:
                await message.edit(embed = generate_message(event_object))
    

async def clean(message: discord.Message, others: bool):
    """Delete all messages in channel for which the event is older than two days.
    
    Optionally also delete every message that doesn't have an event embed
    """

    if len(message.embeds) > 0:
        event_object = get_event_properties(message)
        if event_object.get_event_time()+datetime.timedelta(days=2) < datetime.datetime.now(tz=local_timezone):
            print(event_object.get_event_time())
            await message.delete()
    elif others:
        await message.delete()
