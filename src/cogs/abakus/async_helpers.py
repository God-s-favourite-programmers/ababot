import discord
import datetime
import logging
import pytz

from src.cogs.abakus.helper_functions import get_event_properties, generate_message
from src.cogs.abakus.event import Event

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)

async def get_dm_history(user):
    if user.dm_channel:
        pass
    else:
        await user.create_dm()

    alerts = []
    async for elem in user.dm_channel.history(limit=123):
        if len(elem.embeds) > 0:
            alerts.append(elem)
    return alerts


async def check_message(message: discord.Message, delta) -> None:
    """Retreive the information of an event posting and check if the signup time is within the wanted timedelta."""

    event_object = get_event_properties(message)

    if event_object == None:
        return (False, None, None)

    signupTime = event_object.get_registration_open()
    currentTime = datetime.datetime.now(tz=local_timezone)

    if currentTime+delta >= signupTime:
        msg = generate_message(event_object)
        users = []
        for reaction in message.reactions:
            users.extend(await reaction.users().flatten())
        return (True, users, msg)
    
    else:
        return (False, None, None)


async def remind(user: discord.User, msg: str, client) -> None:
    """Send a message to a user if the exact same message does not allready exist."""

    alerts = await get_dm_history(user)
    alert_titles = [x.embeds[0].title for x in alerts]

    if msg.title not in alert_titles:
        await user.send(embed=msg)
    elif msg.title in alert_titles:
        for message in alerts:
            if len(message.embeds) > 0 and message.embeds[0].title == msg.title and message.author == client.user:
                await message.edit(embed=msg)


async def post(channel, event_object: Event, client) -> None:
    """Post an event in the saved channel or update if it currently exists."""

    msg = generate_message(event_object)

    if msg == None or len(msg) == 0:
        raise ValueError("Message is none")

    messages = []
    async for elem in channel.history(limit=123):
        if len(elem.embeds) > 0 and elem.embeds[0].title not in messages:
            messages.append(elem.embeds[0].title)

    if msg.title not in messages:
        await channel.send(embed=msg)
        logger.debug(f"Event {event_object.get_name()} listed")
    elif msg.title in messages:
        async for elem in channel.history(limit=123):
            if len(elem.embeds) > 0 and elem.embeds[0].title == msg.title and elem.author == client.user:
                await elem.edit(embed = generate_message(event_object))
    
