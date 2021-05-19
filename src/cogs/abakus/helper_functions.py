import datetime
import re
import discord
import pytz
import logging
from src.cogs.abakus.event import Event

local_timezone = pytz.timezone("Europe/Oslo")
logger = logging.getLogger(__name__)

SIGNUP_TIME_FORMAT = "%d/%m kl %H:%M"
START_TIME_FORMAT = "%d/%m kl: %H:%M"

def generate_message(event_object:Event) -> discord.Embed:
    """Return an embed representing the event in the object"""

    if event_object == None:
        raise ValueError("Event object is None")

    time = datetime.datetime.strftime(event_object.get_registration_open(), SIGNUP_TIME_FORMAT)
    startTime = datetime.datetime.strftime(event_object.get_event_time(), START_TIME_FORMAT)

    embed:discord.Embed = discord.Embed(title=event_object.get_name(), url=event_object.get_url(), description=event_object.get_description(), color=0xff0000)
    embed.set_thumbnail(url=event_object.get_thumbnail())
    embed.add_field(name="Registrering", value=time, inline=True)
    embed.add_field(name="Når",value=startTime, inline=True)
    embed.add_field(name="Sted",value=event_object.get_event_location(),inline=True)

    return embed


def get_event_properties(message: discord.Message) -> Event:
    """Return an object representing the event in the message """

    if len(message.embeds) > 0:
        embed = message.embeds[0]
    else:
        return None

    name:str = embed.title
    description:str = embed.description
    url:str = embed.url

    fields = embed.fields
    for field in fields:
        if field.name == "Registrering":
            signupTime:str = field.value
        elif  field.name == "Når":
            startTime:str = field.value
        elif  field.name == "Sted":
            location:str = field.value
    
    thumbnail:str = embed.thumbnail.url
    year = datetime.datetime.now().year
    startTime:datetime.datetime = local_timezone.localize(datetime.datetime.strptime(startTime, START_TIME_FORMAT))
    startTime = startTime.replace(year=year)
    signupTime: datetime.datetime = pytz.utc.localize(datetime.datetime.strptime(signupTime, SIGNUP_TIME_FORMAT))
    signupTime = signupTime.replace(year=year)

    event_options = {
        "name": name,
        "description": description,
        "registration_open": signupTime,
        "event_location": location,
        "event_time": startTime,
        "url": url,
        "thumbnail": thumbnail
    }
    event_object:Event = Event(**event_options)

    return event_object
