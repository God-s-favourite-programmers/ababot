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

    history = await user.dm_channel.history(limit=123).flatten()
    history = [x.content for x in history]
    return history


async def check_message(message: discord.Message) -> None:
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
            return True, reaction.users(), msg


async def remind(self, user: discord.User, msg: str) -> None:
    """Send a message to a user if the exact same message does not allready exist."""

    alerts = await get_dm_history(user)

    if msg not in alerts and len(msg) > 0:
        logger.debug("Direct message sent")
        await user.send(msg)


async def post(channel, event_object: Event) -> None:
    """Post an event in the saved channel if the exact same post does not allready exist."""

    template = "eventTemplate.txt"
    msg = generate_message(event_object, template)

    if msg == None or len(msg) == 0:
        raise ValueError("Message is none")

    messages = [x.content for x in await channel.history(limit=123).flatten()]

    if msg not in messages:
        await channel.send(msg)
        logger.debug(f"Event {event_object.get_name()} listed")
