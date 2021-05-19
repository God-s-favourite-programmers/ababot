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

def generate_message(event_object:Event):
    if event_object == None:
        raise ValueError("Event object is None")
    time = datetime.datetime.strftime(event_object.get_registration_open(), SIGNUP_TIME_FORMAT)
    startTime = datetime.datetime.strftime(event_object.get_event_time(), START_TIME_FORMAT)
    embed=discord.Embed(title=event_object.get_name(), url=event_object.get_url(), description=event_object.get_description(), color=0xff0000)
    embed.set_thumbnail(url=event_object.get_thumbnail())
    embed.add_field(name="Registrering", value=time, inline=True)
    embed.add_field(name="Når",value=startTime, inline=True)
    embed.add_field(name="Sted",value=event_object.get_event_location(),inline=True)
    return embed


def get_event_properties(message: discord.Message) -> Event:
    if len(message.embeds) > 0:
        embed = message.embeds[0]
    else:
        return None

    name = embed.title
    description = embed.description
    url = embed.url
    fields = embed.fields
    for field in fields:
        if field.name == "Registrering":
            signupTime = field.value
        elif  field.name == "Når":
            startTime = field.value
        elif  field.name == "Sted":
            location = field.value
    thumbnail = embed.thumbnail.url
    #if messageSearch:
    #    name = messageSearch.group(1)
    #    description = messageSearch.group(2)
    #    startTime = messageSearch.group(3)
    startTime = local_timezone.localize(
        datetime.datetime.strptime(startTime, START_TIME_FORMAT))
    #    location = messageSearch.group(4)
    #    signupTime = messageSearch.group(5)
    signupTime = local_timezone.localize(
        datetime.datetime.strptime(signupTime, SIGNUP_TIME_FORMAT))
    #    url = messageSearch.group(6)
    event_options = {
        "name": name,
        "description": description,
        "registration_open": signupTime,
        "event_location": location,
        "event_time": startTime,
        "url": url,
        "thumbnail": thumbnail
    }
    event_object = Event(**event_options)
    return event_object
